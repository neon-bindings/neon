//! Asynchronous background _tasks_ that run in the Node thread pool.

use std::marker::{Send, Sized};
use std::mem;
#[cfg(feature = "legacy-runtime")]
use std::os::raw::c_void;

use types::{Value, JsFunction};
use result::JsResult;
use handle::{Handle, Managed};
use context::TaskContext;
#[cfg(feature = "legacy-runtime")]
use context::internal::Env;
use neon_runtime;
#[cfg(feature = "legacy-runtime")]
use neon_runtime::raw;

/// A Rust task that can be executed in a background thread.
pub trait Task: Send + Sized + 'static {
    /// The task's result type, which is sent back to the main thread to communicate a successful result back to JavaScript.
    type Output: Send + 'static;

    /// The task's error type, which is sent back to the main thread to communicate a task failure back to JavaScript.
    type Error: Send + 'static;

    /// The type of JavaScript value that gets produced to the asynchronous callback on the main thread after the task is completed.
    type JsEvent: Value;

    /// Perform the task, producing either a successful `Output` or an unsuccessful `Error`. This method is executed in a background thread as part of libuv's built-in thread pool.
    fn perform(&self) -> Result<Self::Output, Self::Error>;

    /// Convert the result of the task to a JavaScript value to be passed to the asynchronous callback. This method is executed on the main thread at some point after the background task is completed.
    fn complete<'a>(self, cx: TaskContext<'a>, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent>;

    /// Schedule a task to be executed on a background thread.
    ///
    /// `callback` should have the following signature:
    ///
    /// ```js
    /// function callback(err, value) {}
    /// ```
    #[cfg(feature = "legacy-runtime")]
    fn schedule(self, callback: Handle<JsFunction>) {
        schedule_internal(Env::current(), self, callback);
    }

    /// Schedule a task to be executed on a background thread.
    ///
    /// `callback` should have the following signature:
    ///
    /// ```js
    /// function callback(err, value) {}
    /// ```
    #[cfg(feature = "napi-runtime")]
    fn schedule<'a, C: crate::context::Context<'a>>(
        self,
        cx: &mut C,
        callback: Handle<JsFunction>,
    ) {
        let execute = |task: Self| {
            let output = task.perform();

            (task, output)
        };

        let complete = |env, (task, output): (Self, Result<Self::Output, Self::Error>)| {
            let env = unsafe { mem::transmute(env) };

            TaskContext::with(env, |cx| {
                match task.complete(cx, output) {
                    Ok(v) => Some(v.to_raw()),
                    Err(_) => None,
                }
            })
        };

        unsafe {
            neon_runtime::task::schedule(
                cx.env().to_raw(),
                self,
                execute,
                complete,
                callback.to_raw(),
            );
        }
    }
}

#[cfg(feature = "legacy-runtime")]
fn schedule_internal<T: Task>(env: Env, task: T, callback: Handle<JsFunction>) {
    let boxed_task = Box::new(task);
    let task_raw = Box::into_raw(boxed_task);
    let callback_raw = callback.to_raw();
    unsafe {
        neon_runtime::task::schedule(env.to_raw(),
                                     mem::transmute(task_raw),
                                     perform_task::<T>,
                                     complete_task::<T>,
                                     callback_raw);
    }
}

#[cfg(feature = "legacy-runtime")]
unsafe extern "C" fn perform_task<T: Task>(task: *mut c_void) -> *mut c_void {
    let task: Box<T> = Box::from_raw(mem::transmute(task));
    let result = task.perform();
    Box::into_raw(task);
    mem::transmute(Box::into_raw(Box::new(result)))
}

#[cfg(feature = "legacy-runtime")]
unsafe extern "C" fn complete_task<T: Task>(env: *mut c_void, task: *mut c_void, result: *mut c_void, out: &mut raw::Local) {
    let result: Result<T::Output, T::Error> = *Box::from_raw(mem::transmute(result));
    let task: Box<T> = Box::from_raw(mem::transmute(task));
    TaskContext::with(mem::transmute(env), |cx| {
        if let Ok(result) = task.complete(cx, result) {
            *out = result.to_raw();
        }
    })
}
