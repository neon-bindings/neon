use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use neon_runtime::reference;

use context::Context;
use handle::Handle;
use object::Object;
use types::boxed::Finalize;

#[repr(transparent)]
#[derive(Clone)]
struct NapiRef(*mut c_void);

/// `Root<T>` holds a reference to a `JavaScript` object and prevents it from
/// being garbage collected. `Root<T>` may be sent across threads, but the
/// referenced objected may only be accessed on the JavaScript thread that
/// created it.
#[repr(transparent)]
pub struct Root<T> {
    internal: NapiRef,
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
    /// The caller should ensure `Root::into_inner` or `Root::drop` is called
    /// to properly dispose of the `Root<T>`. If the value is dropped without
    /// calling one of these methods, de-allocation will happen via a
    /// slower path. Prior to 0.8, it would actually panic.
    ///
    /// Be careful that you aren't short-circuiting with an early return before
    /// your Root value gets into_inner'd or dropped. The following will cause
    /// the slow path starting with version 0.8 when your error case
    /// gets triggered:
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
}
