#[cfg(feature = "napi-6")]
use std::sync::Arc;

#[cfg(feature = "napi-6")]
use neon_runtime::tsfn::ThreadsafeFunction;
use neon_runtime::{napi, raw};

use crate::context::{internal::Env, Context};
use crate::handle::Managed;
#[cfg(feature = "napi-6")]
use crate::lifecycle::{DropData, InstanceData};
use crate::result::JsResult;
use crate::types::{Handle, Object, Value, ValueInternal};

#[cfg_attr(docsrs, doc(cfg(feature = "promise-api")))]
#[repr(C)]
#[derive(Clone, Copy)]
/// The JavaScript [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise) value.
///
/// [`JsPromise`] may be constructed with [`Context::promise`].
pub struct JsPromise(raw::Local);

impl JsPromise {
    pub(crate) fn new<'a, C: Context<'a>>(cx: &mut C) -> (Deferred, Handle<'a, Self>) {
        let (deferred, promise) = unsafe { napi::promise::create(cx.env().to_raw()) };
        let deferred = Deferred {
            internal: Some(NodeApiDeferred(deferred)),
            #[cfg(feature = "napi-6")]
            drop_queue: InstanceData::drop_queue(cx),
        };

        (deferred, Handle::new_internal(JsPromise(promise)))
    }
}

impl Managed for JsPromise {
    fn to_raw(self) -> raw::Local {
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

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool {
        unsafe { neon_runtime::tag::is_promise(env.to_raw(), other.to_raw()) }
    }
}

impl Value for JsPromise {}

impl Object for JsPromise {}

#[cfg_attr(docsrs, doc(cfg(feature = "promise-api")))]
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
            napi::promise::resolve(cx.env().to_raw(), self.into_inner(), value.to_raw());
        }
    }

    /// Reject a [`JsPromise`] with a JavaScript value
    pub fn reject<'a, V, C>(self, cx: &mut C, value: Handle<V>)
    where
        V: Value,
        C: Context<'a>,
    {
        unsafe {
            napi::promise::reject(cx.env().to_raw(), self.into_inner(), value.to_raw());
        }
    }

    /// Resolve or reject a [`JsPromise`] with the result of a closure
    ///
    /// If the closure throws, the promise will be rejected with the promise
    pub fn settle_with<'a, V, F, C>(self, cx: &mut C, f: F)
    where
        V: Value,
        F: FnOnce(&mut C) -> JsResult<'a, V> + 'static,
        C: Context<'a>,
    {
        match cx.try_catch_internal(f) {
            Ok(value) => self.resolve(cx, value),
            Err(err) => self.reject(cx, err),
        }
    }

    fn into_inner(mut self) -> napi::Deferred {
        self.internal.take().unwrap().0
    }
}

#[repr(transparent)]
pub(crate) struct NodeApiDeferred(napi::Deferred);

unsafe impl Send for NodeApiDeferred {}

#[cfg(feature = "napi-6")]
impl NodeApiDeferred {
    pub(crate) unsafe fn leaked(self, env: raw::Env) {
        napi::promise::reject_err_message(
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
