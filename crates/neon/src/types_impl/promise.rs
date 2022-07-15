use std::ptr;

use crate::{
    context::{internal::Env, Context},
    handle::{internal::TransparentNoCopyWrapper, Handle, Managed},
    object::Object,
    result::JsResult,
    sys::{self, no_panic::FailureBoundary, raw},
    types::{private::ValueInternal, Value},
};

#[cfg(feature = "napi-4")]
use crate::{
    context::TaskContext,
    event::{Channel, JoinHandle, SendError},
};

#[cfg(feature = "napi-6")]
use crate::{
    lifecycle::{DropData, InstanceData},
    sys::tsfn::ThreadsafeFunction,
};

#[cfg(all(feature = "napi-5", feature = "futures"))]
use {
    crate::context::internal::ContextInternal,
    crate::event::{JoinError, SendThrow},
    crate::result::NeonResult,
    crate::types::{JsFunction, JsValue},
    std::future::Future,
    std::pin::Pin,
    std::sync::Mutex,
    std::task::{self, Poll},
    tokio::sync::oneshot,
};

#[cfg(any(feature = "napi-6", all(feature = "napi-5", feature = "futures")))]
use std::sync::Arc;

const BOUNDARY: FailureBoundary = FailureBoundary {
    both: "A panic and exception occurred while resolving a `neon::types::Deferred`",
    exception: "An exception occurred while resolving a `neon::types::Deferred`",
    panic: "A panic occurred while resolving a `neon::types::Deferred`",
};

#[derive(Debug)]
#[repr(transparent)]
#[cfg_attr(
    feature = "promise-api",
    deprecated = "`promise-api` feature has no impact and may be removed"
)]
/// The JavaScript [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise) value.
///
/// [`JsPromise`] may be constructed with [`Context::promise`].
pub struct JsPromise(raw::Local);

impl JsPromise {
    pub(crate) fn new<'a, C: Context<'a>>(cx: &mut C) -> (Deferred, Handle<'a, Self>) {
        let (deferred, promise) = unsafe { sys::promise::create(cx.env().to_raw()) };
        let deferred = Deferred {
            internal: Some(NodeApiDeferred(deferred)),
            #[cfg(feature = "napi-6")]
            drop_queue: InstanceData::drop_queue(cx),
        };

        (deferred, Handle::new_internal(JsPromise(promise)))
    }

    /// Creates a new `Promise` immediately resolved with the given value. If the value is a
    /// `Promise` or a then-able, it will be flattened.
    ///
    /// `JsPromise::resolve` is useful to ensure a value that might not be a `Promise` or
    /// might not be a native promise is converted to a `Promise` before use.
    pub fn resolve<'a, C: Context<'a>, T: Value>(cx: &mut C, value: Handle<T>) -> Handle<'a, Self> {
        let (deferred, promise) = cx.promise();
        deferred.resolve(cx, value);
        promise
    }

    /// Creates a nwe `Promise` immediately rejected with the given error.
    pub fn reject<'a, C: Context<'a>, E: Value>(cx: &mut C, err: Handle<E>) -> Handle<'a, Self> {
        let (deferred, promise) = cx.promise();
        deferred.reject(cx, err);
        promise
    }

    #[cfg(all(feature = "napi-5", feature = "futures"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "napi-5", feature = "futures"))))]
    /// Creates a [`Future`](std::future::Future) that can be awaited to receive the result of a
    /// JavaScript `Promise`.
    ///
    /// A callback must be provided that maps a `Result` representing the resolution or rejection of
    /// the `Promise` and returns a value as the `Future` output.
    ///
    /// _Note_: Unlike `Future`, `Promise` are eagerly evaluated and so are `JsFuture`.
    pub fn to_future<'a, O, C, F>(&self, cx: &mut C, f: F) -> NeonResult<JsFuture<O>>
    where
        O: Send + 'static,
        C: Context<'a>,
        F: FnOnce(TaskContext, Result<Handle<JsValue>, Handle<JsValue>>) -> NeonResult<O>
            + Send
            + 'static,
    {
        let then = self.get::<JsFunction, _, _>(cx, "then")?;
        let catch = self.get::<JsFunction, _, _>(cx, "catch")?;

        let (tx, rx) = oneshot::channel();
        let take_state = {
            // Note: If this becomes a bottleneck, `unsafe` could be used to avoid it.
            // The promise spec guarantees that it will only be used once.
            let state = Arc::new(Mutex::new(Some((f, tx))));

            move || {
                state
                    .lock()
                    .ok()
                    .and_then(|mut lock| lock.take())
                    // This should never happen because `self` is a native `Promise`
                    // and settling multiple times is a violation of the spec.
                    .expect("Attempted to settle JsFuture multiple times")
            }
        };

        let resolve = JsFunction::new(cx, {
            let take_state = take_state.clone();

            move |mut cx| {
                let (f, tx) = take_state();
                let v = cx.argument::<JsValue>(0)?;

                TaskContext::with_context(cx.env(), move |cx| {
                    // Error indicates that the `Future` has already dropped; ignore
                    let _ = tx.send(f(cx, Ok(v)).map_err(Into::into));
                });

                Ok(cx.undefined())
            }
        })?;

        let reject = JsFunction::new(cx, {
            move |mut cx| {
                let (f, tx) = take_state();
                let v = cx.argument::<JsValue>(0)?;

                TaskContext::with_context(cx.env(), move |cx| {
                    // Error indicates that the `Future` has already dropped; ignore
                    let _ = tx.send(f(cx, Err(v)).map_err(Into::into));
                });

                Ok(cx.undefined())
            }
        })?;

        then.exec(cx, Handle::new_internal(Self(self.0)), [resolve.upcast()])?;
        catch.exec(cx, Handle::new_internal(Self(self.0)), [reject.upcast()])?;

        Ok(JsFuture { rx })
    }
}

unsafe impl TransparentNoCopyWrapper for JsPromise {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl Managed for JsPromise {
    fn to_raw(&self) -> raw::Local {
        self.0
    }

    fn from_raw(_env: Env, h: raw::Local) -> Self {
        Self(h)
    }
}

impl ValueInternal for JsPromise {
    fn name() -> String {
        "Promise".to_string()
    }

    fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool {
        unsafe { sys::tag::is_promise(env.to_raw(), other.to_raw()) }
    }
}

impl Value for JsPromise {}

impl Object for JsPromise {}

/// [`Deferred`] is a handle that can be used to resolve or reject a [`JsPromise`]
///
/// It is recommended to settle a [`Deferred`] with [`Deferred::settle_with`] to ensure
/// exceptions are caught.
///
/// On Node-API versions less than 6, dropping a [`Deferred`] without settling will
/// cause a panic. On Node-API 6+, the associated [`JsPromise`] will be automatically
/// rejected.
pub struct Deferred {
    internal: Option<NodeApiDeferred>,
    #[cfg(feature = "napi-6")]
    drop_queue: Arc<ThreadsafeFunction<DropData>>,
}

impl Deferred {
    /// Resolve a [`JsPromise`] with a JavaScript value
    pub fn resolve<'a, V, C>(self, cx: &mut C, value: Handle<V>)
    where
        V: Value,
        C: Context<'a>,
    {
        unsafe {
            sys::promise::resolve(cx.env().to_raw(), self.into_inner(), value.to_raw());
        }
    }

    /// Reject a [`JsPromise`] with a JavaScript value
    pub fn reject<'a, V, C>(self, cx: &mut C, value: Handle<V>)
    where
        V: Value,
        C: Context<'a>,
    {
        unsafe {
            sys::promise::reject(cx.env().to_raw(), self.into_inner(), value.to_raw());
        }
    }

    #[cfg(feature = "napi-4")]
    #[cfg_attr(docsrs, doc(cfg(feature = "napi-4")))]
    /// Settle the [`JsPromise`] by sending a closure across a [`Channel`][`crate::event::Channel`]
    /// to be executed on the main JavaScript thread.
    ///
    /// Usage is identical to [`Deferred::settle_with`].
    ///
    /// Returns a [`SendError`][crate::event::SendError] if sending the closure to the main JavaScript thread fails.
    /// See [`Channel::try_send`][crate::event::Channel::try_send] for more details.
    pub fn try_settle_with<V, F>(
        self,
        channel: &Channel,
        complete: F,
    ) -> Result<JoinHandle<()>, SendError>
    where
        V: Value,
        F: FnOnce(TaskContext) -> JsResult<V> + Send + 'static,
    {
        channel.try_send(move |cx| {
            self.try_catch_settle(cx, complete);
            Ok(())
        })
    }

    #[cfg(feature = "napi-4")]
    #[cfg_attr(docsrs, doc(cfg(feature = "napi-4")))]
    /// Settle the [`JsPromise`] by sending a closure across a [`Channel`][crate::event::Channel]
    /// to be executed on the main JavaScript thread.
    ///
    /// Panics if there is a libuv error.
    ///
    /// ```
    /// # use neon::prelude::*;
    /// # fn example(mut cx: FunctionContext) -> JsResult<JsPromise> {
    /// let channel = cx.channel();
    /// let (deferred, promise) = cx.promise();
    ///
    /// deferred.settle_with(&channel, move |mut cx| Ok(cx.number(42)));
    ///
    /// # Ok(promise)
    /// # }
    /// ```
    pub fn settle_with<V, F>(self, channel: &Channel, complete: F) -> JoinHandle<()>
    where
        V: Value,
        F: FnOnce(TaskContext) -> JsResult<V> + Send + 'static,
    {
        self.try_settle_with(channel, complete).unwrap()
    }

    pub(crate) fn try_catch_settle<'a, C, V, F>(self, cx: C, f: F)
    where
        C: Context<'a>,
        V: Value,
        F: FnOnce(C) -> JsResult<'a, V>,
    {
        unsafe {
            BOUNDARY.catch_failure(
                cx.env().to_raw(),
                Some(self.into_inner()),
                move |_| match f(cx) {
                    Ok(value) => value.to_raw(),
                    Err(_) => ptr::null_mut(),
                },
            );
        }
    }

    pub(crate) fn into_inner(mut self) -> sys::Deferred {
        self.internal.take().unwrap().0
    }
}

#[repr(transparent)]
pub(crate) struct NodeApiDeferred(sys::Deferred);

unsafe impl Send for NodeApiDeferred {}

#[cfg(feature = "napi-6")]
impl NodeApiDeferred {
    pub(crate) unsafe fn leaked(self, env: raw::Env) {
        sys::promise::reject_err_message(
            env,
            self.0,
            "`neon::types::Deferred` was dropped without being settled",
        );
    }
}

impl Drop for Deferred {
    #[cfg(not(feature = "napi-6"))]
    fn drop(&mut self) {
        // If `None`, the `Deferred` has already been settled
        if self.internal.is_none() {
            return;
        }

        // Destructors are called during stack unwinding, prevent a double
        // panic and instead prefer to leak.
        if std::thread::panicking() {
            eprintln!("Warning: neon::types::JsPromise leaked during a panic");
            return;
        }

        // Only panic if the event loop is still running
        if let Ok(true) = crate::context::internal::IS_RUNNING.try_with(|v| *v.borrow()) {
            panic!("Must settle a `neon::types::JsPromise` with `neon::types::Deferred`");
        }
    }

    #[cfg(feature = "napi-6")]
    fn drop(&mut self) {
        // If `None`, the `Deferred` has already been settled
        if let Some(internal) = self.internal.take() {
            let _ = self.drop_queue.call(DropData::Deferred(internal), None);
        }
    }
}

#[cfg(all(feature = "napi-5", feature = "futures"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "napi-5", feature = "futures"))))]
/// A [`Future`](std::future::Future) created from a [`JsPromise`].
///
/// Unlike typical `Future`, `JsFuture` are eagerly executed because they
/// are backed by a `Promise`.
pub struct JsFuture<T> {
    // `Err` is always `Throw`, but `Throw` cannot be sent across threads
    rx: oneshot::Receiver<Result<T, SendThrow>>,
}

#[cfg(all(feature = "napi-5", feature = "futures"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "napi-5", feature = "futures"))))]
impl<T> Future for JsFuture<T> {
    type Output = Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(result) => {
                // Flatten `Result<Result<T, SendThrow>, RecvError>` by mapping to
                // `Result<T, JoinError>`. This can be simplified by replacing the
                // closure with a try-block after stabilization.
                // https://doc.rust-lang.org/beta/unstable-book/language-features/try-blocks.html
                let get_result = move || Ok(result??);

                Poll::Ready(get_result())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
