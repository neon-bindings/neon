use std::{mem::MaybeUninit, ptr};

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn new(out: &mut Local, env: Env, data: *const u8, len: i32) -> bool {
    let status = napi::create_string_utf8(env, data as *const _, len as usize, out);

    status == napi::Status::Ok
}

pub unsafe fn utf8_len(env: Env, value: Local) -> usize {
    let mut len = MaybeUninit::uninit();
    let status = napi::get_value_string_utf8(env, value, ptr::null_mut(), 0, len.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);

    len.assume_init()
}

pub unsafe fn data(env: Env, out: *mut u8, len: usize, value: Local) -> usize {
    let mut read = MaybeUninit::uninit();
    let status = napi::get_value_string_utf8(env, value, out as *mut _, len, read.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);

    read.assume_init()
}

pub unsafe fn utf16_len(env: Env, value: Local) -> usize {
    let mut len = MaybeUninit::uninit();
    let status = napi::get_value_string_utf16(env, value, ptr::null_mut(), 0, len.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);

    len.assume_init()
}

pub unsafe fn data_utf16(env: Env, out: *mut u16, len: usize, value: Local) -> usize {
    let mut read = MaybeUninit::uninit();
    let status = napi::get_value_string_utf16(env, value, out, len, read.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);

    read.assume_init()
}

pub unsafe fn run_script(out: &mut Local, env: Env, value: Local) -> bool {
    let status = napi::run_script(env, value, out as *mut _);

    status == napi::Status::Ok
}
