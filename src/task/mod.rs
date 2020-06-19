//! Asynchronous background _tasks_ that run in the Node thread pool.

use std::marker::{Send, Sized};
use std::os::raw::c_void;

use types::{Value, JsFunction};
use result::JsResult;
use handle::{Handle, Managed};
use context::{Context, TaskContext};
use neon_runtime;
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
    fn schedule(self, callback: Handle<JsFunction>) {
        schedule(move || {
            let result = self.perform();

            move |cx| self.complete(cx, result)
        }, callback);
    }
}

pub struct TaskBuilder<'c, C, Perform> {
    // Placeholder for future methods and N-API implementation
    _context: &'c mut C,
    perform: Perform,
}

impl<'c, C, Perform> TaskBuilder<'c, C, Perform> {
    pub(crate) fn new(context: &'c mut C, perform: Perform) -> Self {
        TaskBuilder {
            _context: context,
            perform,
        }
    }
}

impl<'a, 'c, C, Perform, Complete, Output> TaskBuilder<'c, C, Perform>
where
    C: Context<'a>,
    Perform: FnOnce() -> Complete + Send + 'static,
    Complete: FnOnce(TaskContext) -> JsResult<Output> + Send + 'static,
    Output: Value,
{
    pub fn schedule(self, callback: Handle<JsFunction>) {
        let Self { perform, .. } = self;

        schedule(perform, callback);
    }
}

fn schedule<Perform, Complete, Output>(
    perform: Perform,
    callback: Handle<JsFunction>,
)
where
    Perform: FnOnce() -> Complete + Send + 'static,
    Complete: FnOnce(TaskContext) -> JsResult<Output> + Send + 'static,
    Output: Value,
{
    let data = Box::into_raw(Box::new(perform));

    unsafe {
        neon_runtime::task::schedule(
            data as *mut _,
            perform_task::<Perform, Complete>,
            complete_task::<Complete, Output>,
            callback.to_raw(),
        );
    }
}

unsafe extern "C" fn perform_task<Perform, Output>(
    perform: *mut c_void,
) -> *mut c_void
where
    Perform: FnOnce() -> Output,
{
    let perform = Box::from_raw(perform as *mut Perform);
    let result = perform();

    Box::into_raw(Box::new(result)) as *mut _
}

unsafe extern "C" fn complete_task<Complete, Output>(
    complete: *mut c_void,
    out: &mut raw::Local,
)
where
    Complete: FnOnce(TaskContext) -> JsResult<Output>,
    Output: Value,
{
    let complete = *Box::from_raw(complete as *mut Complete);

    TaskContext::with(|cx| {
        if let Ok(result) = complete(cx) {
            *out = result.to_raw();
        }
    })
}
