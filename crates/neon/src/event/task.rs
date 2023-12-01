use std::{panic::resume_unwind, thread};

use crate::{
    context::{internal::Env, Context, TaskContext},
    handle::Handle,
    result::{JsResult, NeonResult},
    sys::{async_work, raw},
    types::{Deferred, JsPromise, Value},
};

/// Node asynchronous task builder
///
/// ```
/// # use neon::prelude::*;
/// fn greet(mut cx: FunctionContext) -> JsResult<JsPromise> {
///     let name = cx.argument::<JsString>(0)?.value(&mut cx);
///
///     let promise = cx
///         .task(move || format!("Hello, {}!", name))
///         .promise(move |mut cx, greeting| Ok(cx.string(greeting)));
///
///     Ok(promise)
/// }
/// ```
pub struct TaskBuilder<'cx, C, E> {
    cx: &'cx mut C,
    execute: E,
}

impl<'a: 'cx, 'cx, C, O, E> TaskBuilder<'cx, C, E>
where
    C: Context<'a>,
    O: Send + 'static,
    E: FnOnce() -> O + Send + 'static,
{
    /// Construct a new task builder from an `execute` callback that can be
    /// scheduled to execute on the Node worker pool
    pub fn new(cx: &'cx mut C, execute: E) -> Self {
        Self { cx, execute }
    }

    /// Schedules a task to execute on the Node worker pool, executing the
    /// `complete` callback on the JavaScript main thread with the result
    /// of the `execute` callback
    pub fn and_then<F>(self, complete: F)
    where
        F: FnOnce(TaskContext, O) -> NeonResult<()> + 'static,
    {
        let env = self.cx.env();
        let execute = self.execute;

        schedule(env, execute, complete);
    }

    /// Schedules a task to execute on the Node worker pool and returns a
    /// promise that is resolved with the value from the `complete` callback.
    ///
    /// The `complete` callback will execute on the JavaScript main thread and
    /// is passed the return value from `execute`. If the `complete` callback
    /// throws, the promise will be rejected with the exception
    pub fn promise<V, F>(self, complete: F) -> Handle<'a, JsPromise>
    where
        V: Value,
        F: FnOnce(TaskContext, O) -> JsResult<V> + 'static,
    {
        let env = self.cx.env();
        let (deferred, promise) = JsPromise::new(self.cx);
        let execute = self.execute;

        schedule_promise(env, execute, complete, deferred);

        promise
    }
}

// Schedule a task to execute on the Node worker pool
fn schedule<I, O, D>(env: Env, input: I, data: D)
where
    I: FnOnce() -> O + Send + 'static,
    O: Send + 'static,
    D: FnOnce(TaskContext, O) -> NeonResult<()> + 'static,
{
    unsafe {
        async_work::schedule(env.to_raw(), input, execute::<I, O>, complete::<O, D>, data);
    }
}

fn execute<I, O>(input: I) -> O
where
    I: FnOnce() -> O + Send + 'static,
    O: Send + 'static,
{
    input()
}

fn complete<O, D>(env: raw::Env, output: thread::Result<O>, callback: D)
where
    O: Send + 'static,
    D: FnOnce(TaskContext, O) -> NeonResult<()> + 'static,
{
    let output = output.unwrap_or_else(|panic| {
        // If a panic was caught while executing the task on the Node Worker
        // pool, resume panicking on the main JavaScript thread
        resume_unwind(panic)
    });

    TaskContext::with_context(env.into(), move |cx| {
        let _ = callback(cx, output);
    });
}

// Schedule a task to execute on the Node worker pool and settle a `Promise` with the result
fn schedule_promise<I, O, D, V>(env: Env, input: I, complete: D, deferred: Deferred)
where
    I: FnOnce() -> O + Send + 'static,
    O: Send + 'static,
    D: FnOnce(TaskContext, O) -> JsResult<V> + 'static,
    V: Value,
{
    unsafe {
        async_work::schedule(
            env.to_raw(),
            input,
            execute::<I, O>,
            complete_promise::<O, D, V>,
            (complete, deferred),
        );
    }
}

fn complete_promise<O, D, V>(
    env: raw::Env,
    output: thread::Result<O>,
    (complete, deferred): (D, Deferred),
) where
    O: Send + 'static,
    D: FnOnce(TaskContext, O) -> JsResult<V> + 'static,
    V: Value,
{
    let env = env.into();

    TaskContext::with_context(env, move |cx| {
        deferred.try_catch_settle(cx, move |cx| {
            let output = output.unwrap_or_else(|panic| resume_unwind(panic));

            complete(cx, output)
        })
    });
}
