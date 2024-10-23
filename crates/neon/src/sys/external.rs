use std::mem::MaybeUninit;

use super::{
    bindings as napi,
    debug_send_wrapper::DebugSendWrapper,
    raw::{Env, Local},
};

/// `finalize_external` is invoked immediately before a `napi_external` is garbage collected
extern "C" fn finalize_external<T: 'static>(
    env: Env,
    // Raw pointer to a `Box<T>` stored by a `napi_external`
    data: *mut std::ffi::c_void,
    // Pointer to a Rust `fn` stored in the `hint` parameter of a `napi_external` called
    // with the contents of `data` immediately before the value is garbage collected.
    hint: *mut std::ffi::c_void,
) {
    unsafe {
        let data = Box::<DebugSendWrapper<T>>::from_raw(data as *mut _);
        let finalizer: fn(Env, T) = std::mem::transmute(hint as *const ());

        finalizer(env, data.take());
    }
}

/// Returns a pointer to data stored in a `napi_external`
/// Safety: `deref` must only be called with `napi_external` created by that
/// module. Calling `deref` with an external created by another native module,
/// even another neon module, is undefined behavior.
/// <https://github.com/neon-bindings/neon/issues/591>
pub unsafe fn deref<T: 'static>(env: Env, local: Local) -> Option<*const T> {
    let mut result = MaybeUninit::uninit();
    let () = napi::typeof_value(env, local, result.as_mut_ptr()).unwrap();

    let result = result.assume_init();

    // Ensure we have an external
    if result != napi::ValueType::External {
        return None;
    }

    // As a future improvement, this could be done with a dynamic symbol check instead of
    // relying on the Node-API version compatibility at compile time.
    #[cfg(feature = "napi-8")]
    // Check the external came from this module
    if !super::tag::check_object_type_tag(env, local, &crate::MODULE_TAG) {
        return None;
    }

    let mut result = MaybeUninit::uninit();
    let () = napi::get_value_external(env, local, result.as_mut_ptr()).unwrap();

    let v = result.assume_init();
    let v = &**v.cast_const().cast::<DebugSendWrapper<T>>() as *const T;

    Some(v)
}

/// Creates a `napi_external` from a Rust type
pub unsafe fn create<T: 'static>(env: Env, v: T, finalizer: fn(Env, T)) -> Local {
    let v = Box::new(DebugSendWrapper::new(v));
    let mut result = MaybeUninit::uninit();

    let status = napi::create_external(
        env,
        Box::into_raw(v) as *mut _,
        Some(finalize_external::<T>),
        // Casting to `*const ()` is required to ensure the correct layout
        // https://rust-lang.github.io/unsafe-code-guidelines/layout/function-pointers.html
        finalizer as *const () as *mut _,
        result.as_mut_ptr(),
    );

    // `napi_create_external` will only fail if the VM is in a throwing state
    // or shutting down.
    let () = status.unwrap();

    let external = result.assume_init();

    #[cfg(feature = "napi-8")]
    // Tag the object as coming from this module
    super::tag::type_tag_object(env, external, &crate::MODULE_TAG);

    external
}
