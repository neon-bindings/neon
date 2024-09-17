use std::{mem::MaybeUninit, panic::Location, ptr};

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn is_throwing(env: Env) -> bool {
    let mut b: MaybeUninit<bool> = MaybeUninit::zeroed();

    napi::is_exception_pending(env, b.as_mut_ptr()).unwrap();

    b.assume_init()
}

pub unsafe fn catch_error(env: Env, error: *mut Local) -> bool {
    if !is_throwing(env) {
        return false;
    }

    napi::get_and_clear_last_exception(env, error).unwrap();

    true
}

pub unsafe fn clear_exception(env: Env) {
    let mut result = MaybeUninit::uninit();
    napi::is_exception_pending(env, result.as_mut_ptr()).unwrap();

    if !result.assume_init() {
        return;
    }

    let mut result = MaybeUninit::uninit();
    napi::get_and_clear_last_exception(env, result.as_mut_ptr()).unwrap();
}

pub unsafe fn throw(env: Env, val: Local) {
    napi::throw(env, val).unwrap();
}

pub unsafe fn new_error(env: Env, out: &mut Local, msg: Local) {
    let mut result = MaybeUninit::uninit();
    napi::create_error(env, ptr::null_mut(), msg, result.as_mut_ptr()).unwrap();

    *out = result.assume_init();
}

pub unsafe fn new_type_error(env: Env, out: &mut Local, msg: Local) {
    let mut result = MaybeUninit::uninit();
    napi::create_type_error(env, ptr::null_mut(), msg, result.as_mut_ptr()).unwrap();

    *out = result.assume_init();
}

pub unsafe fn new_range_error(env: Env, out: &mut Local, msg: Local) {
    let mut result = MaybeUninit::uninit();
    napi::create_range_error(env, ptr::null_mut(), msg, result.as_mut_ptr()).unwrap();

    *out = result.assume_init();
}

pub unsafe fn throw_error_from_utf8(env: Env, msg: *const u8, len: i32) {
    let mut out = MaybeUninit::uninit();

    napi::create_string_utf8(env, msg as *const _, len as usize, out.as_mut_ptr()).unwrap();

    let mut err = MaybeUninit::uninit();
    napi::create_error(env, ptr::null_mut(), out.assume_init(), err.as_mut_ptr()).unwrap();

    throw(env, err.assume_init());
}

#[track_caller]
pub(super) unsafe fn fatal_error(message: &str) -> ! {
    let location = Location::caller().to_string();

    napi::fatal_error(
        location.as_ptr().cast(),
        location.len(),
        message.as_ptr().cast(),
        message.len(),
    );

    unreachable!("Expected napi_fatal_error to exit the process")
}
