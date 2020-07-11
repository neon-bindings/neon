use std::mem::MaybeUninit;
use raw::{Env, Local};

use nodejs_sys as napi;

pub unsafe extern "C" fn throw(env: Env, val: Local) {
    let status = napi::napi_throw(env, val);

    assert_eq!(status, napi::napi_status::napi_ok);
}

pub unsafe extern "C" fn new_error(_out: &mut Local, _msg: Local) { unimplemented!() }

pub unsafe extern "C" fn new_type_error(_out: &mut Local, _msg: Local) { unimplemented!() }

pub unsafe extern "C" fn new_range_error(_out: &mut Local, _msg: Local) { unimplemented!() }

pub unsafe extern "C" fn throw_error_from_utf8(_msg: *const u8, _len: i32) { unimplemented!() }

pub unsafe extern "C" fn is_throwing(env: Env) -> bool {
    let mut b: MaybeUninit<bool> = MaybeUninit::zeroed();

    let status = napi::napi_is_exception_pending(env, b.as_mut_ptr());

    assert_eq!(status, napi::napi_status::napi_ok);

    b.assume_init()
}

pub unsafe extern "C" fn catch_error(env: Env, error: *mut Local) -> bool {
    if !is_throwing(env) {
        return false;
    }

    let status = napi::napi_get_and_clear_last_exception(env, error);

    assert_eq!(status, napi::napi_status::napi_ok);

    true
}
