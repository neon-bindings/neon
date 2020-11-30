use std::mem::MaybeUninit;
use std::ptr;

use crate::napi::bindings as napi;
use crate::raw::{Env, Local};

pub unsafe fn is_throwing(env: Env) -> bool {
    let mut b: MaybeUninit<bool> = MaybeUninit::zeroed();

    let status = napi::is_exception_pending(env, b.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);

    b.assume_init()
}

pub unsafe fn catch_error(env: Env, error: *mut Local) -> bool {
    if !is_throwing(env) {
        return false;
    }

    let status = napi::get_and_clear_last_exception(env, error);

    assert_eq!(status, napi::Status::Ok);

    true
}

pub unsafe fn clear_exception(env: Env) {
    let mut result = MaybeUninit::uninit();
    let status = napi::is_exception_pending(env, result.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);

    if !result.assume_init() {
        return;
    }

    let mut result = MaybeUninit::uninit();
    let status = napi::get_and_clear_last_exception(env, result.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);
}

pub unsafe fn throw(env: Env, val: Local) {
    let status = napi::throw(env, val);

    assert_eq!(status, napi::Status::Ok);
}

pub unsafe fn new_error(env: Env, out: &mut Local, msg: Local) {
    let mut result = MaybeUninit::uninit();
    let status = napi::create_error(
        env,
        ptr::null_mut(),
        msg,
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::Status::Ok);

    *out = result.assume_init();
}

pub unsafe fn new_type_error(env: Env, out: &mut Local, msg: Local) {
    let mut result = MaybeUninit::uninit();
    let status = napi::create_type_error(
        env,
        ptr::null_mut(),
        msg,
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::Status::Ok);

    *out = result.assume_init();
}

pub unsafe fn new_range_error(env: Env, out: &mut Local, msg: Local) {
    let mut result = MaybeUninit::uninit();
    let status = napi::create_range_error(
        env,
        ptr::null_mut(),
        msg,
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::Status::Ok);

    *out = result.assume_init();
}

pub unsafe fn throw_error_from_utf8(env: Env, msg: *const u8, len: i32) {
    let mut out = MaybeUninit::uninit();
    let status = napi::create_string_utf8(
        env,
        msg as *const i8,
        len as usize,
        out.as_mut_ptr(),
    );

    assert_eq!(status, napi::Status::Ok);

    let mut err = MaybeUninit::uninit();
    let status = napi::create_error(
        env,
        ptr::null_mut(),
        out.assume_init(),
        err.as_mut_ptr(),
    );

    assert_eq!(status, napi::Status::Ok);

    throw(env, err.assume_init());
}
