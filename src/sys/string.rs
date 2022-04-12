use std::{mem::MaybeUninit, ptr};

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn new(out: &mut Local, env: Env, data: *const u8, len: i32) -> bool {
    let status = napi::create_string_utf8(env, data as *const _, len as usize, out);

    status == napi::Status::Ok
}

pub unsafe fn utf8_len(env: Env, value: Local) -> isize {
    let mut len = MaybeUninit::uninit();
    let status = napi::get_value_string_utf8(env, value, ptr::null_mut(), 0, len.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);

    len.assume_init() as isize
}

pub unsafe fn data(env: Env, out: *mut u8, len: isize, value: Local) -> isize {
    let mut read = MaybeUninit::uninit();
    let status =
        napi::get_value_string_utf8(env, value, out as *mut _, len as usize, read.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);

    read.assume_init() as isize
}

pub unsafe fn run_script(out: &mut Local, env: Env, value: Local) -> bool {
    let status = napi::run_script(env, value, out as *mut _);

    status == napi::Status::Ok
}
