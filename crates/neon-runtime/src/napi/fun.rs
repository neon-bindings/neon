//! Facilities for working with JS functions.

use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::ptr;

use crate::napi::bindings as napi;
use crate::raw::{Env, Local};

pub unsafe fn new<F>(env: Env, name: &str, callback: F) -> Result<Local, napi::Status>
where
    F: Fn(Env, napi::CallbackInfo) -> Local + Send + 'static,
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

    if status == napi::Status::PendingException {
        Box::from_raw(data);

        return Err(status);
    }

    assert_eq!(status, napi::Status::Ok);

    let out = out.assume_init();

    #[cfg(feature = "napi-5")]
    {
        unsafe extern "C" fn drop_function<F>(
            _env: Env,
            _finalize_data: *mut c_void,
            finalize_hint: *mut c_void,
        ) {
            Box::from_raw(finalize_hint.cast::<F>());
        }

        let status = napi::add_finalizer(
            env,
            out,
            ptr::null_mut(),
            Some(drop_function::<F>),
            data.cast(),
            ptr::null_mut(),
        );

        assert_eq!(status, napi::Status::Ok);
    }

    Ok(out)
}

unsafe extern "C" fn call_boxed<F>(env: Env, info: napi::CallbackInfo) -> Local
where
    F: Fn(Env, napi::CallbackInfo) -> Local + Send + 'static,
{
    let mut data = MaybeUninit::uninit();
    let status = napi::get_cb_info(
        env,
        info,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        data.as_mut_ptr(),
    );

    assert_eq!(status, napi::Status::Ok);

    let callback = &*data.assume_init().cast::<F>();

    callback(env, info)
}

pub unsafe fn call(
    out: &mut Local,
    env: Env,
    fun: Local,
    this: Local,
    argc: i32,
    argv: *const c_void,
) -> bool {
    let status = napi::call_function(
        env,
        this,
        fun,
        argc as usize,
        argv as *const _,
        out as *mut _,
    );

    status == napi::Status::Ok
}

pub unsafe fn construct(
    out: &mut Local,
    env: Env,
    fun: Local,
    argc: i32,
    argv: *const c_void,
) -> bool {
    let status = napi::new_instance(env, fun, argc as usize, argv as *const _, out as *mut _);

    status == napi::Status::Ok
}
