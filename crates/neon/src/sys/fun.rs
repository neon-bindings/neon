//! Facilities for working with JS functions.

use std::{mem::MaybeUninit, os::raw::c_void, ptr};

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn new<F>(env: Env, name: &str, callback: F) -> Result<Local, napi::Status>
where
    F: Fn(Env, napi::CallbackInfo) -> Local + 'static,
{
    let mut out = MaybeUninit::uninit();
    let data = Box::into_raw(Box::new(callback));
    let status = napi::create_function(
        env,
        name.as_ptr().cast(),
        name.len(),
        Some(call_boxed::<F>),
        data.cast(),
        out.as_mut_ptr(),
    );

    let () = match status {
        Err(err @ napi::Status::PendingException) => {
            drop(Box::from_raw(data));

            return Err(err);
        }
        status => status.unwrap(),
    };

    let out = out.assume_init();

    #[cfg(feature = "napi-5")]
    {
        unsafe extern "C" fn drop_function<F>(
            _env: Env,
            _finalize_data: *mut c_void,
            finalize_hint: *mut c_void,
        ) {
            drop(Box::from_raw(finalize_hint.cast::<F>()));
        }

        let status = napi::add_finalizer(
            env,
            out,
            ptr::null_mut(),
            Some(drop_function::<F>),
            data.cast(),
            ptr::null_mut(),
        );

        // If adding the finalizer fails the closure will leak, but it would
        // be unsafe to drop it because there's no guarantee V8 won't use the
        // pointer.
        let () = status.unwrap();
    }

    Ok(out)
}

// C ABI compatible function for invoking a boxed closure from the data field
// of a Node-API JavaScript function
unsafe extern "C" fn call_boxed<F>(env: Env, info: napi::CallbackInfo) -> Local
where
    F: Fn(Env, napi::CallbackInfo) -> Local + 'static,
{
    let mut data = MaybeUninit::uninit();
    let () = napi::get_cb_info(
        env,
        info,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        data.as_mut_ptr(),
    )
    .unwrap();

    let callback = &*data.assume_init().cast::<F>();

    callback(env, info)
}

pub unsafe fn construct(
    out: &mut Local,
    env: Env,
    fun: Local,
    argc: usize,
    argv: *const c_void,
) -> bool {
    let status = napi::new_instance(env, fun, argc, argv as *const _, out as *mut _);

    status.is_ok()
}
