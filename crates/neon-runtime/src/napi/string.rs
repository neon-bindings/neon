use std::mem::MaybeUninit;
use std::ptr;

use crate::raw::{Env, Local};
use crate::napi::bindings as napi;

pub unsafe fn new(out: &mut Local, env: Env, data: *const u8, len: i32) -> bool {
    let status = napi::create_string_utf8(
        env,
        data as *const i8,
        len as usize,
        out,
    );

    status == napi::Status::Ok
}

pub unsafe extern "C" fn utf8_len(env: Env, value: Local) -> isize {
    let mut len = MaybeUninit::uninit();
    let status = napi::get_value_string_utf8(
        env,
        value,
        ptr::null_mut(),
        0,
        len.as_mut_ptr(),
    );

    assert_eq!(status, napi::Status::Ok);

    len.assume_init() as isize
}

pub unsafe extern "C" fn data(env: Env, out: *mut u8, len: isize, value: Local) -> isize {
    let mut read = MaybeUninit::uninit();
    let status = napi::get_value_string_utf8(
        env,
        value,
        out as *mut i8,
        len as usize,
        read.as_mut_ptr(),
    );

    assert_eq!(status, napi::Status::Ok);

    read.assume_init() as isize
}
