use neon_runtime::{async_work, raw};

#[cfg(feature = "promise-api")]
use crate::context::internal::ContextInternal;
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

type ExecuteOutput<O, F> = (O, F);
#[cfg(feature = "promise-api")]
type PromiseOutput<O, F> = (O, F, Deferred);

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
        F: FnOnce(&mut TaskContext, O) -> NeonResult<()> + Send + 'static,
    {
        let env = self.cx.env();
        let execute = self.execute;

        // Wrap the user provided `execute` callback with a callback that
        // runs it but, also passes the user's `complete` callback to the
        // static `complete` function.
        let execute = move || (execute(), complete);

        schedule(env, execute);
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
        for<'b> F: FnOnce(&mut TaskContext<'b>, O) -> JsResult<'b, V> + Send + 'static,
    {
        let env = self.cx.env();
        let (deferred, promise) = JsPromise::new(self.cx);
        let execute = self.execute;

        // Wrap the user provided `execute` callback with a callback that runs it
        // but, also passes the user's `complete` callback and `Deferred` into the
        // static `complete` function.
        let execute = move || (execute(), complete, deferred);

        schedule_promise(env, execute);

        promise
    }
}

// Schedule a task to execute on the Node worker pool
fn schedule<O, F, I>(env: Env, input: I)
where
    O: Send + 'static,
    F: FnOnce(&mut TaskContext, O) -> NeonResult<()> + Send + 'static,
    I: FnOnce() -> ExecuteOutput<O, F> + Send + 'static,
{
    unsafe {
        async_work::schedule(
            env.to_raw(),
            input,
            execute::<I, ExecuteOutput<O, F>>,
            complete::<O, F>,
        );
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
fn complete<O, F>(env: raw::Env, (output, complete): ExecuteOutput<O, F>)
where
    O: Send + 'static,
    F: FnOnce(&mut TaskContext, O) -> NeonResult<()> + Send + 'static,
{
    let env = unsafe { std::mem::transmute(env) };

    TaskContext::with_context(env, move |mut cx| {
        let _ = complete(&mut cx, output);
    });
}

#[cfg(feature = "promise-api")]
// Schedule a task to execute on the Node worker pool and settle a `Promise` with the result
fn schedule_promise<O, V, F, I>(env: Env, input: I)
where
    O: Send + 'static,
    V: Value,
    for<'b> F: FnOnce(&mut TaskContext<'b>, O) -> JsResult<'b, V> + Send + 'static,
    I: FnOnce() -> PromiseOutput<O, F> + Send + 'static,
{
    unsafe {
        async_work::schedule(
            env.to_raw(),
            input,
            execute::<I, PromiseOutput<O, F>>,
            complete_promise::<O, V, F>,
        );
    }
}

#[cfg(feature = "promise-api")]
// Unpack an `(output, complete, deferred)` tuple returned by `execute` and settle the
// deferred with the result of passing `output` to the `complete` callback
fn complete_promise<O, V, F>(env: raw::Env, (output, complete, deferred): PromiseOutput<O, F>)
where
    O: Send + 'static,
    V: Value,
    for<'b> F: FnOnce(&mut TaskContext<'b>, O) -> JsResult<'b, V> + Send + 'static,
{
    let env = unsafe { std::mem::transmute(env) };

    TaskContext::with_context(env, move |mut cx| {
        match cx.try_catch_internal(move |cx| complete(cx, output)) {
            Ok(value) => deferred.resolve(&mut cx, value),
            Err(err) => deferred.reject(&mut cx, err),
        }
    });
}
