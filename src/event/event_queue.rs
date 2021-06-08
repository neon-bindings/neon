use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
    // We hold an extra reference to `state` in `try_send` so that we could
    // unref the tsfn during the same UV tick if the state is guaranteed to be
    // dropped before the `try_send`'s closure invocation.
    state: Arc<ChannelState>,
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
        let state = ChannelState {
            shared: Arc::new(ChannelSharedState::new(cx)),
            has_ref: AtomicBool::new(true),
        };

        Self {
            state: Arc::new(state),
        }
    }

    /// Allow the Node event loop to exit while this `Channel` exists.
    /// _Idempotent_
    pub fn unref<'a, C: Context<'a>>(&mut self, cx: &mut C) -> &mut Self {
        self.state.unref(cx);
        self
    }

    /// Prevent the Node event loop from exiting while this `Channel` exists. (Default)
    /// _Idempotent_
    pub fn reference<'a, C: Context<'a>>(&mut self, cx: &mut C) -> &mut Self {
        self.state.reference(cx);
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
        let state = self.state.clone();

        let callback = Box::new(move |env| {
            let env = unsafe { std::mem::transmute(env) };

            // Note: It is sufficient to use `TaskContext`'s `InheritedHandleScope` because
            // N-API creates a `HandleScope` before calling the callback.
            TaskContext::with_context(env, move |mut cx| {
                // No one else, but us holding the state alive.
                if Arc::strong_count(&state) == 1 {
                    state.unref(&mut cx);
                }

                let _ = f(cx);
            });
        });

        self.state
            .shared
            .tsfn
            .read()
            .unwrap()
            .call(callback, None)
            .map_err(|_| SendError)
    }

    /// Returns a boolean indicating if this `Channel` will prevent the Node event
    /// loop from exiting.
    pub fn has_ref(&self) -> bool {
        self.state.has_ref.load(Ordering::Relaxed)
    }
}

impl Clone for Channel {
    /// Returns a clone of the Channel instance that shares the internal
    /// unbounded queue with the original channel. Scheduling callbacks on the
    /// same queue is faster than using separate channels, but might lead to
    /// starvation if one of the threads posts significantly more callbacks on
    /// the channel than the other one.
    fn clone(&self) -> Self {
        return Self {
            state: Arc::new((*self.state).clone()),
        };
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
    shared: Arc<ChannelSharedState>,
    has_ref: AtomicBool,
}

impl ChannelState {
    fn reference<'a, C: Context<'a>>(&self, cx: &mut C) {
        // Already referenced
        if self.has_ref.swap(true, Ordering::Relaxed) {
            return;
        }

        self.shared.reference(cx);
    }

    fn unref<'a, C: Context<'a>>(&self, cx: &mut C) {
        // Already unreferenced
        if !self.has_ref.swap(false, Ordering::Relaxed) {
            return;
        }

        self.shared.unref(cx);
    }
}

impl Clone for ChannelState {
    fn clone(&self) -> Self {
        // Not referenced, we can simply clone the fields
        if !self.has_ref.load(Ordering::Relaxed) {
            return Self {
                shared: self.shared.clone(),
                has_ref: AtomicBool::new(false),
            };
        }

        let shared = Arc::clone(&self.shared);

        // Only need to increase the ref count since the tsfn is already referenced
        shared.ref_count.fetch_add(1, Ordering::Relaxed);

        Self {
            shared,
            has_ref: AtomicBool::new(true),
        }
    }
}

impl Drop for ChannelState {
    fn drop(&mut self) {
        // It was only us who kept the `ChannelState` alive. No need to unref
        // the `tsfn`, because it is going to be dropped once this function
        // returns.
        if Arc::strong_count(&self.shared) == 1 {
            return;
        }

        // Not a referenced event queue
        if !self.has_ref.swap(false, Ordering::Relaxed) {
            return;
        }

        // The ChannelState is dropped on a worker thread. We have to `unref`
        // the tsfn on the UV thread after all pending closures.
        let shared = Arc::clone(&self.shared);

        let callback = Box::new(move |env| {
            let env = unsafe { std::mem::transmute(env) };

            // Note: It is sufficient to use `TaskContext`'s `InheritedHandleScope` because
            // N-API creates a `HandleScope` before calling the callback.
            TaskContext::with_context(env, move |mut cx| {
                shared.unref(&mut cx);
            });
        });

        self.shared
            .tsfn
            .read()
            .unwrap()
            .call(callback, None)
            .map_err(|_| SendError)
            .unwrap();
    }
}

struct ChannelSharedState {
    tsfn: RwLock<ThreadsafeFunction<Callback>>,
    ref_count: AtomicUsize,
}

impl ChannelSharedState {
    fn new<'a, C: Context<'a>>(cx: &mut C) -> Self {
        let tsfn = unsafe { ThreadsafeFunction::new(cx.env().to_raw(), Self::callback) };
        Self {
            tsfn: RwLock::new(tsfn),
            ref_count: AtomicUsize::new(1),
        }
    }

    fn reference<'a, C: Context<'a>>(&self, cx: &mut C) {
        if self.ref_count.fetch_add(1, Ordering::Relaxed) != 0 {
            return;
        }

        // Critical section, avoid panicking
        {
            let mut tsfn = self.tsfn.write().unwrap();
            unsafe { tsfn.reference(cx.env().to_raw()) }
        }
        .unwrap();
    }

    fn unref<'a, C: Context<'a>>(&self, cx: &mut C) {
        if self.ref_count.fetch_sub(1, Ordering::Relaxed) != 1 {
            return;
        }

        // Critical section, avoid panicking
        {
            let mut tsfn = self.tsfn.write().unwrap();
            unsafe { tsfn.unref(cx.env().to_raw()) }
        }
        .unwrap();
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
