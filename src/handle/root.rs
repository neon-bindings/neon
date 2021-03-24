use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::sync::Arc;

use neon_runtime::reference;
#[cfg(feature = "napi-6")]
use neon_runtime::tsfn::ThreadsafeFunction;

use context::Context;
use handle::Handle;
#[cfg(feature = "napi-6")]
use lifecycle::InstanceData;
use object::Object;
use types::boxed::Finalize;

#[repr(transparent)]
#[derive(Clone)]
pub(crate) struct NapiRef(*mut c_void);

// # Safety
// `NapiRef` are reference counted types that allow references to JavaScript objects
// to outlive a `Context` (`napi_env`). Since access is serialized by obtaining a
// `Context`, they are both `Send` and `Sync`.
// https://nodejs.org/api/n-api.html#n_api_references_to_objects_with_a_lifespan_longer_than_that_of_the_native_method
unsafe impl Send for NapiRef {}
unsafe impl Sync for NapiRef {}

/// `Root<T>` holds a reference to a `JavaScript` object and prevents it from
/// being garbage collected. `Root<T>` may be sent across threads, but the
/// referenced objected may only be accessed on the JavaScript thread that
/// created it.
pub struct Root<T> {
    internal: NapiRef,
    #[cfg(feature = "napi-6")]
    drop_queue: Arc<ThreadsafeFunction<NapiRef>>,
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
    ///
    /// For example, early return with a ? or an explicit return before freeing
    /// a `Root` will trigger the slow path:
    /// ```
    /// # use neon::prelude::*;
    /// # fn create_log_entry(a: &str, b: &str) -> Result<String, String> { unimplemented!() }
    /// # fn my_neon_function(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// # let id_generator = "";
    /// let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
    /// let my_log = match (create_log_entry(&id_generator, "log-emitter")) {
    ///                Err(_err) => {
    ///                  return cx.throw_error("Couldn't construct log");
    ///                },
    ///                Ok(log) => log,
    /// };
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    /// The solution in the original case for this was to bind the callback
    /// after the fallible code, right before spawning an async task.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, value: &T) -> Self {
        let env = cx.env().to_raw();
        let internal = unsafe { reference::new(env, value.to_raw()) };

        Self {
            internal: NapiRef(internal as *mut _),
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
        let internal = self.internal.0 as *mut _;

        unsafe {
            reference::reference(env.to_raw(), internal);
        };

        Self {
            internal: self.internal.clone(),
            #[cfg(feature = "napi-6")]
            drop_queue: Arc::clone(&self.drop_queue),
            _phantom: PhantomData,
        }
    }

    /// Safely drop a `Root<T>` without returning the referenced JavaScript
    /// object.
    pub fn drop<'a, C: Context<'a>>(self, cx: &mut C) {
        let env = cx.env().to_raw();
        let internal = ManuallyDrop::new(self).internal.0 as *mut _;

        unsafe {
            reference::unreference(env, internal);
        }
    }

    /// Return the referenced JavaScript object and allow it to be garbage collected
    pub fn into_inner<'a, C: Context<'a>>(self, cx: &mut C) -> Handle<'a, T> {
        let env = cx.env();
        let internal = ManuallyDrop::new(self).internal.0 as *mut _;

        let local = unsafe { reference::get(env.to_raw(), internal) };

        unsafe {
            reference::unreference(env.to_raw(), internal);
        }

        Handle::new_internal(T::from_raw(env, local))
    }

    /// Access the inner JavaScript object without consuming the `Root`
    /// This method aliases the reference without changing the reference count. It
    /// can be used in place of a clone immediately followed by a call to `into_inner`.
    pub fn to_inner<'a, C: Context<'a>>(&self, cx: &mut C) -> Handle<'a, T> {
        let env = cx.env();
        let local = unsafe { reference::get(env.to_raw(), self.internal.0 as *mut _) };

        Handle::new_internal(T::from_raw(env, local))
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
        // Destructors are called during stack unwinding, prevent a double
        // panic and instead prefer to leak.
        if std::thread::panicking() {
            eprintln!("Warning: neon::sync::Root leaked during a panic");
            return;
        }

        // Only panic if the event loop is still running
        if let Ok(true) = crate::context::internal::IS_RUNNING.try_with(|v| *v.borrow()) {
            panic!(
                "Must call `into_inner` or `drop` on `Root` \
                https://docs.rs/neon/latest/neon/sync/index.html#drop-safety"
            );
        }
    }

    #[cfg(feature = "napi-6")]
    fn drop(&mut self) {
        let _ = self.drop_queue.call(self.internal.clone(), None);
    }
}
