use std::sync::{Arc, RwLock};

use neon_runtime::raw::Env;
use neon_runtime::tsfn::ThreadsafeFunction;

use crate::context::{Context, TaskContext};
use crate::result::NeonResult;

type Callback = Box<dyn FnOnce(Env) + Send + 'static>;

/// Channel for scheduling Rust closures to execute on the JavaScript main thread.
///
/// Cloning a `Channel` will create a new channel that shares a backing queue for
/// events.
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
///     // These types (`f64`, `Root<JsFunction>`, `Channel`) may all be sent
///     // across threads.
///     let n = cx.argument::<JsNumber>(0)?.value(&mut cx);
///     let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
///     let channel = cx.channel();
///
///     // Spawn a thread to complete the execution. This will _not_ block the
///     // JavaScript event loop.
///     std::thread::spawn(move || {
///         let result = fibonacci(n);
///
///         // Send a closure as a task to be executed by the JavaScript event
///         // loop. This _will_ block the event loop while executing.
///         channel.send(move |mut cx| {
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

pub struct Channel {
    state: Arc<RwLock<ChannelState>>,
    has_ref: bool,
}

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Channel")
    }
}

impl Channel {
    /// Creates an unbounded channel for scheduling closures on the JavaScript
    /// main thread
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> Self {
        let tsfn = unsafe { ThreadsafeFunction::new(cx.env().to_raw(), Self::callback) };
        let state = ChannelState { tsfn, ref_count: 1 };

        Self {
            state: Arc::new(RwLock::new(state)),
            has_ref: true,
        }
    }

    /// Allow the Node event loop to exit while this `Channel` exists.
    /// _Idempotent_
    pub fn unref<'a, C: Context<'a>>(&mut self, cx: &mut C) -> &mut Self {
        // Already unreferenced
        if !self.has_ref {
            return self;
        }

        ChannelState::unref(cx, &self.state);

        self.has_ref = false;
        self
    }

    /// Prevent the Node event loop from exiting while this `Channel` exists. (Default)
    /// _Idempotent_
    pub fn reference<'a, C: Context<'a>>(&mut self, cx: &mut C) -> &mut Self {
        // Already referenced
        if self.has_ref {
            return self;
        }

        ChannelState::reference(cx, &self.state);

        self.has_ref = true;
        self
    }

    /// Schedules a closure to execute on the JavaScript thread that created this Channel
    /// Panics if there is a libuv error
    pub fn send<F>(&self, f: F)
    where
        F: FnOnce(TaskContext) -> NeonResult<()> + Send + 'static,
    {
        self.try_send(f).unwrap()
    }

    /// Schedules a closure to execute on the JavaScript thread that created this Channel
    /// Returns an `Error` if the task could not be scheduled.
    pub fn try_send<F>(&self, f: F) -> Result<(), SendError>
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

        self.state
            .read()
            .unwrap()
            .tsfn
            .call(callback, None)
            .map_err(|_| SendError)
    }

    /// Returns a boolean indicating if this `Channel` will prevent the Node event
    /// loop from exiting.
    pub fn has_ref(&self) -> bool {
        self.has_ref
    }

    // Monomorphized trampoline funciton for calling the user provided closure
    fn callback(env: Option<Env>, callback: Callback) {
        if let Some(env) = env {
            callback(env);
        } else {
            crate::context::internal::IS_RUNNING.with(|v| {
                *v.borrow_mut() = false;
            });
        }
    }
}

impl Clone for Channel {
    fn clone(&self) -> Self {
        // Not referenced, we can simply clone the fields
        if !self.has_ref {
            return Self {
                state: Arc::clone(&self.state),
                has_ref: false,
            };
        }

        let mut state = self.state.write().unwrap();

        // Only need to increase the ref count since the tsfn is already referenced
        debug_assert!(state.ref_count > 0);
        state.ref_count += 1;

        Self {
            state: Arc::clone(&self.state),
            has_ref: true,
        }
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        // Not a referenced event queue
        if !self.has_ref {
            return;
        }

        let state = Arc::clone(&self.state);

        self.send(move |mut cx| {
            ChannelState::unref(&mut cx, &state);
            Ok(())
        });
    }
}

/// Error indicating that a closure was unable to be scheduled to execute on the event loop.
pub struct SendError;

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SendError")
    }
}

impl std::fmt::Debug for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for SendError {}

struct ChannelState {
    tsfn: ThreadsafeFunction<Callback>,
    ref_count: usize,
}

impl ChannelState {
    fn reference<'a, C: Context<'a>>(cx: &mut C, state: &RwLock<ChannelState>) {
        // Critical section, avoid panicking
        {
            let mut state = state.write().unwrap();

            state.ref_count += 1;

            if state.ref_count == 1 {
                unsafe { state.tsfn.reference(cx.env().to_raw()) }
            } else {
                Ok(())
            }
        }
        .unwrap();
    }

    fn unref<'a, C: Context<'a>>(cx: &mut C, state: &RwLock<ChannelState>) {
        // Critical section, avoid panicking
        {
            let mut state = state.write().unwrap();

            debug_assert!(state.ref_count >= 1);
            state.ref_count -= 1;

            if state.ref_count == 0 {
                unsafe { state.tsfn.unref(cx.env().to_raw()) }
            } else {
                Ok(())
            }
        }
        .unwrap();
    }
}
