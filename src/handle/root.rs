use std::{ffi::c_void, marker::PhantomData};

use crate::{
    context::Context,
    handle::Handle,
    object::Object,
    sys::{raw, reference},
    types::boxed::Finalize,
};

#[cfg(feature = "napi-6")]
use {
    crate::{
        lifecycle::{DropData, InstanceData, InstanceId},
        sys::tsfn::ThreadsafeFunction,
    },
    std::sync::Arc,
};

#[cfg(not(feature = "napi-6"))]
use std::thread::{self, ThreadId};

#[cfg(not(feature = "napi-6"))]
type InstanceId = ThreadId;

#[repr(transparent)]
#[derive(Clone)]
pub(crate) struct NapiRef(*mut c_void);

impl NapiRef {
    /// # Safety
    /// Must only be used from the same module context that created the reference
    pub(crate) unsafe fn unref(self, env: raw::Env) {
        reference::unreference(env, self.0.cast());
    }
}

// # Safety
// `NapiRef` are reference counted types that allow references to JavaScript objects
// to outlive a `Context` (`napi_env`). Since access is serialized by obtaining a
// `Context`, they are both `Send` and `Sync`.
// https://nodejs.org/api/n-api.html#n_api_references_to_objects_with_a_lifespan_longer_than_that_of_the_native_method
unsafe impl Send for NapiRef {}

unsafe impl Sync for NapiRef {}

/// A thread-safe handle that holds a reference to a JavaScript object and
/// prevents it from being garbage collected.
///
/// A `Root<T>` may be sent across threads, but the referenced object may
/// only be accessed on the JavaScript thread that created it.
pub struct Root<T> {
    // `Option` is used to skip `Drop` when `Root::drop` or `Root::into_inner` is used.
    // It will *always* be `Some` when a user is interacting with `Root`.
    internal: Option<NapiRef>,
    instance_id: InstanceId,
    #[cfg(feature = "napi-6")]
    drop_queue: Arc<ThreadsafeFunction<DropData>>,
    _phantom: PhantomData<T>,
}

impl<T> std::fmt::Debug for Root<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Root<{}>", std::any::type_name::<T>())
    }
}

// `Root` are intended to be `Send` and `Sync`
// Safety: `Root` contains two types. A `NapiRef` which is `Send` and `Sync` and a
// `PhantomData` that does not impact the safety.
unsafe impl<T> Send for Root<T> {}

unsafe impl<T> Sync for Root<T> {}

#[cfg(feature = "napi-6")]
fn instance_id<'a, C: Context<'a>>(cx: &mut C) -> InstanceId {
    InstanceData::id(cx)
}

#[cfg(not(feature = "napi-6"))]
fn instance_id<'a, C: Context<'a>>(_: &mut C) -> InstanceId {
    thread::current().id()
}

impl<T: Object> Root<T> {
    /// Create a reference to a JavaScript object. The object will not be
    /// garbage collected until the `Root` is dropped. A `Root<T>` may only
    /// be dropped on the JavaScript thread that created it.
    ///
    /// The caller _should_ ensure `Root::into_inner` or `Root::drop` is called
    /// to properly dispose of the `Root<T>`. If the value is dropped without
    /// calling one of these methods:
    /// * N-API < 6, Neon will `panic` to notify of the leak
    /// * N-API >= 6, Neon will drop from a global queue at a runtime cost
    pub fn new<'a, C: Context<'a>>(cx: &mut C, value: &T) -> Self {
        let env = cx.env().to_raw();
        let internal = unsafe { reference::new(env, value.to_raw()) };

        Self {
            internal: Some(NapiRef(internal as *mut _)),
            instance_id: instance_id(cx),
            #[cfg(feature = "napi-6")]
            drop_queue: InstanceData::drop_queue(cx),
            _phantom: PhantomData,
        }
    }

    /// Clone a reference to the contained JavaScript object. This method can
    /// be considered identical to the following:
    /// ```
    /// # use neon::prelude::*;
    /// # fn my_neon_function(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// # let root = cx.argument::<JsObject>(0)?.root(&mut cx);
    /// let inner = root.into_inner(&mut cx);
    /// let cloned = inner.root(&mut cx);
    /// let root = inner.root(&mut cx);
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    pub fn clone<'a, C: Context<'a>>(&self, cx: &mut C) -> Self {
        let env = cx.env();
        let internal = self.as_napi_ref(cx).0 as *mut _;

        unsafe {
            reference::reference(env.to_raw(), internal);
        };

        Self {
            internal: self.internal.clone(),
            instance_id: instance_id(cx),
            #[cfg(feature = "napi-6")]
            drop_queue: Arc::clone(&self.drop_queue),
            _phantom: PhantomData,
        }
    }

    /// Safely drop a `Root<T>` without returning the referenced JavaScript
    /// object.
    pub fn drop<'a, C: Context<'a>>(self, cx: &mut C) {
        let env = cx.env().to_raw();

        unsafe {
            self.into_napi_ref(cx).unref(env);
        }
    }

    /// Return the referenced JavaScript object and allow it to be garbage collected
    pub fn into_inner<'a, C: Context<'a>>(self, cx: &mut C) -> Handle<'a, T> {
        let env = cx.env();
        let internal = self.into_napi_ref(cx);
        let local = unsafe { reference::get(env.to_raw(), internal.0.cast()) };

        unsafe {
            internal.unref(env.to_raw());
        }

        Handle::new_internal(T::from_raw(env, local))
    }

    /// Access the inner JavaScript object without consuming the `Root`
    /// This method aliases the reference without changing the reference count. It
    /// can be used in place of a clone immediately followed by a call to `into_inner`.
    pub fn to_inner<'a, C: Context<'a>>(&self, cx: &mut C) -> Handle<'a, T> {
        let env = cx.env();
        let local = unsafe { reference::get(env.to_raw(), self.as_napi_ref(cx).0 as *mut _) };

        Handle::new_internal(T::from_raw(env, local))
    }

    fn as_napi_ref<'a, C: Context<'a>>(&self, cx: &mut C) -> &NapiRef {
        if self.instance_id != instance_id(cx) {
            panic!("Attempted to dereference a `neon::handle::Root` from the wrong module ");
        }

        self.internal
            .as_ref()
            // `unwrap` will not `panic` because `internal` will always be `Some`
            // until the `Root` is consumed.
            .unwrap()
    }

    fn into_napi_ref<'a, C: Context<'a>>(mut self, cx: &mut C) -> NapiRef {
        let reference = self.as_napi_ref(cx).clone();
        // This uses `as_napi_ref` instead of `Option::take` for the instance id safety check
        self.internal = None;
        reference
    }
}

// Allows putting `Root<T>` directly in a container that implements `Finalize`
// For example, `Vec<Root<T>>` or `JsBox`.
impl<T: Object> Finalize for Root<T> {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        self.drop(cx);
    }
}

impl<T> Drop for Root<T> {
    #[cfg(not(feature = "napi-6"))]
    fn drop(&mut self) {
        // If `None`, the `NapiRef` has already been manually dropped
        if self.internal.is_none() {
            return;
        }

        // Destructors are called during stack unwinding, prevent a double
        // panic and instead prefer to leak.
        if std::thread::panicking() {
            eprintln!("Warning: neon::handle::Root leaked during a panic");
            return;
        }

        // Only panic if the event loop is still running
        if let Ok(true) = crate::context::internal::IS_RUNNING.try_with(|v| *v.borrow()) {
            panic!("Must call `into_inner` or `drop` on `neon::handle::Root`");
        }
    }

    #[cfg(feature = "napi-6")]
    fn drop(&mut self) {
        // If `None`, the `NapiRef` has already been manually dropped
        if let Some(internal) = self.internal.take() {
            let _ = self.drop_queue.call(DropData::Ref(internal), None);
        }
    }
}
