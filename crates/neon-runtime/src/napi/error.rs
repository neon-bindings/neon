use std::mem::MaybeUninit;
use std::ptr;

use nodejs_sys as napi;

use raw::{Env, Local};

pub unsafe fn clear_exception(env: Env) {
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_is_exception_pending(env, result.as_mut_ptr());

    assert_eq!(status, napi::napi_status::napi_ok);

    if !result.assume_init() {
        return;
    }

    let mut result = MaybeUninit::uninit();
    let status = napi::napi_get_and_clear_last_exception(env, result.as_mut_ptr());

    assert_eq!(status, napi::napi_status::napi_ok);
}

pub unsafe fn throw(env: Env, val: Local) {
    let status = napi::napi_throw(env, val);

    assert_eq!(status, napi::napi_status::napi_ok);
}

pub unsafe fn new_error(env: Env, out: &mut Local, msg: Local) {
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_create_error(
        env,
        ptr::null_mut(),
        msg,
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::napi_status::napi_ok);

    *out = result.assume_init();
}

pub unsafe fn new_type_error(env: Env, out: &mut Local, msg: Local) {
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_create_type_error(
        env,
        ptr::null_mut(),
        msg,
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::napi_status::napi_ok);

    *out = result.assume_init();
}

pub unsafe fn new_range_error(env: Env, out: &mut Local, msg: Local) {
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_create_range_error(
        env,
        ptr::null_mut(),
        msg,
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::napi_status::napi_ok);

    *out = result.assume_init();
}

pub unsafe fn throw_error_from_utf8(env: Env, msg: *const u8, len: i32) {
    let mut out = MaybeUninit::uninit();
    let status = napi::napi_create_string_utf8(
        env,
        msg as *const i8,
        len as usize,
        out.as_mut_ptr(),
    );

    assert_eq!(status, napi::napi_status::napi_ok);

    let mut err = MaybeUninit::uninit();
    let status = napi::napi_create_error(
        env,
        ptr::null_mut(),
        out.assume_init(),
        err.as_mut_ptr(),
    );

    assert_eq!(status, napi::napi_status::napi_ok);

    throw(env, err.assume_init());
}
