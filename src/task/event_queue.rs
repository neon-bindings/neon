use neon_runtime::raw::Env;
use neon_runtime::tsfn::ThreadsafeFunction;

use context::{Context, TaskContext};
use result::NeonResult;

type Callback = Box<dyn FnOnce(Env) + Send + 'static>;

/// Queue for scheduling Rust closures to execute on tge JavaScript main thread
///
/// # Example
///
/// The following example spawns a standard Rust thread to complete a computation
/// and calls back to a JavaScript function asynchronously with the result.
///
/// ```
/// # use neon::prelude::*;
/// # fn fibonacci(_: f64) -> f64 { todo!() }
/// fn async_fibonacci(mut cx: FunctionContext) -> JsResult<JsUndefined> {
///     // These types (`f64`, `Root<JsFunction>`, `EventQueue`) may all be sent
///     // across threads.
///     let n = cx.argument::<JsNumber>(0)?.value(&mut cx);
///     let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
///     let queue = cx.queue();
///
///     // Spawn a thread to complete the execution. This will _not_ block the
///     // JavaScript event loop.
///     std::thread::spawn(move || {
///         let result = fibonacci(n);
///
///         // Send a closure as a task to be executed by the JavaScript event
///         // queue. This _will_ block the event queue while executing.
///         queue.send(move |mut cx| {
///             let callback = callback.into_inner(&mut cx);
///             let this = cx.undefined();
///             let null = cx.null();
///             let args = vec![
///                 cx.null().upcast::<JsValue>(),
///                 cx.number(result).upcast(),
///             ];
/// 
///             callback.call(&mut cx, this, args)?;
///
///             Ok(())
///         });
///     });
/// 
///     Ok(cx.undefined())
/// }
/// ```

pub struct EventQueue {
    tsfn: ThreadsafeFunction<Callback>,
    has_ref: bool,
}

impl std::fmt::Debug for EventQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EventQueue")
    }
}

impl EventQueue {
    /// Creates an unbounded queue for scheduling closures on the JavaScript
    /// main thread
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> Self {
        let tsfn = unsafe {
            ThreadsafeFunction::new(
                cx.env().to_raw(),
                Self::callback,
            )
        };

        Self {
            tsfn,
            has_ref: true,
        }
    }

    /// Allow the Node event loop to exit while this `EventQueue` exists.
    /// _Idempotent_
    pub fn unref<'a, C: Context<'a>>(&mut self, cx: &mut C) -> &mut Self {
        self.has_ref = false;

        unsafe {
            self.tsfn.unref(cx.env().to_raw())
        }

        self
    }

    /// Prevent the Node event loop from exiting while this `EventQueue` exists. (Default)
    /// _Idempotent_
    pub fn reference<'a, C: Context<'a>>(&mut self, cx: &mut C) -> &mut Self {
        self.has_ref = true;

        unsafe {
            self.tsfn.reference(cx.env().to_raw())
        }

        self
    }

    /// Schedules a closure to execute on the JavaScript thread that created this EventQueue
    /// Panics if there is a libuv error
    pub fn send<F>(&self, f: F)
    where
        F: FnOnce(TaskContext) -> NeonResult<()> + Send + 'static,
    {
        self.try_send(f).unwrap()
    }

    /// Schedules a closure to execute on the JavaScript thread that created this EventQueue
    /// Returns an `Error` if the task could not be scheduled.
    pub fn try_send<F>(&self, f: F) -> Result<(), EventQueueError>
    where
        F: FnOnce(TaskContext) -> NeonResult<()> + Send + 'static,
    {
        let callback = Box::new(move |env| {
            let env = unsafe { std::mem::transmute(env) };

            // Note: It is sufficient to use `TaskContext`'s `InheritedHandleScope` because
            // N-API creates a `HandleScope` before calling the callback.
            TaskContext::with_context(env, move |cx| {
                let _ = f(cx);
            });
        });

        self.tsfn
            .call(callback, None)
            .map_err(|_| EventQueueError)
    }

    /// Returns a boolean indicating if this `EventQueue` will prevent the Node event
    /// queue from exiting.
    pub fn has_ref(&self) -> bool {
        self.has_ref
    }

    // Monomorphized trampoline funciton for calling the user provided closure
    fn callback(env: Env, callback: Callback) {
        callback(env)
    }
}

/// Error indicating that a closure was unable to be scheduled to execute on the event queue
pub struct EventQueueError;

impl std::fmt::Display for EventQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventQueueError")
    }
}

impl std::fmt::Debug for EventQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for EventQueueError {}
