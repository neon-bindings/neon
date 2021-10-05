use neon_runtime::{async_work, raw};

use crate::context::{internal::Env, Context, TaskContext};
#[cfg(feature = "promise-api")]
use crate::handle::Handle;
#[cfg(feature = "promise-api")]
use crate::result::JsResult;
use crate::result::NeonResult;
#[cfg(feature = "promise-api")]
use crate::types::Value;
#[cfg(feature = "promise-api")]
use crate::types::{Deferred, JsPromise};

#[cfg_attr(docsrs, doc(cfg(feature = "task-api")))]
/// Node asynchronous task builder
///
/// ```
/// # #[cfg(feature = "promise-api")] {
/// # use neon::prelude::*;
/// fn greet(mut cx: FunctionContext) -> JsResult<JsPromise> {
///     let name = cx.argument::<JsString>(0)?.value(&mut cx);
///
///     let promise = cx
///         .task(move || format!("Hello, {}!", name))
///         .promise(move |cx, greeting| Ok(cx.string(greeting)));
///
///     Ok(promise)
/// }
/// # }
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
        F: FnOnce(TaskContext, O) -> NeonResult<()> + Send + 'static,
    {
        let env = self.cx.env();
        let execute = self.execute;

        schedule(env, execute, complete);
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "promise-api")))]
    #[cfg(feature = "promise-api")]
    /// Schedules a task to execute on the Node worker pool and returns a
    /// promise that is resolved with the value from the `complete` callback.
    ///
    /// The `complete` callback will execute on the JavaScript main thread and
    /// is passed the return value from `execute`. If the `complete` callback
    /// throws, the promise will be rejected with the exception
    pub fn promise<V, F>(self, complete: F) -> Handle<'a, JsPromise>
    where
        V: Value,
        F: FnOnce(TaskContext, O) -> JsResult<V> + Send + 'static,
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
    D: FnOnce(TaskContext, O) -> NeonResult<()> + Send + 'static,
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

// Unpack an `(output, complete)` tuple returned by `execute` and execute
// `complete` with the `output` argument
fn complete<O, D>(env: raw::Env, output: O, callback: D)
where
    O: Send + 'static,
    D: FnOnce(TaskContext, O) -> NeonResult<()> + Send + 'static,
{
    TaskContext::with_context(env.into(), move |cx| {
        let _ = callback(cx, output);
    });
}

#[cfg(feature = "promise-api")]
// Schedule a task to execute on the Node worker pool and settle a `Promise` with the result
fn schedule_promise<I, O, D, V>(env: Env, input: I, complete: D, deferred: Deferred)
where
    I: FnOnce() -> O + Send + 'static,
    O: Send + 'static,
    D: FnOnce(TaskContext, O) -> JsResult<V> + Send + 'static,
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

#[cfg(feature = "promise-api")]
// Unpack an `(output, complete, deferred)` tuple returned by `execute` and settle the
// deferred with the result of passing `output` to the `complete` callback
fn complete_promise<O, D, V>(env: raw::Env, output: O, (complete, deferred): (D, Deferred))
where
    O: Send + 'static,
    D: FnOnce(TaskContext, O) -> JsResult<V> + Send + 'static,
    V: Value,
{
    let env = env.into();

    TaskContext::with_context(env, move |cx| unsafe {
        match env.try_catch(move || complete(cx, output)) {
            Ok(value) => {
                neon_runtime::promise::resolve(env.to_raw(), deferred.into_inner(), value.to_raw());
            }
            Err(err) => {
                neon_runtime::promise::reject(env.to_raw(), deferred.into_inner(), err);
            }
        }
    });
}
