use std::{mem::MaybeUninit, ptr};

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn new(out: &mut Local, env: Env, data: *const u8, len: i32) -> bool {
    let status = napi::create_string_utf8(env, data as *const _, len as usize, out);

    status.is_ok()
}

/// Creates a string suitable for use as a property key. With Node-API 10 the string is
/// internalized, which lets the engine look up properties faster; otherwise this is
/// equivalent to [`new`].
pub unsafe fn new_property_key(out: &mut Local, env: Env, data: *const u8, len: i32) -> bool {
    property_key_utf8(env, data, len, out).is_ok()
}

#[cfg(feature = "napi-10")]
pub(super) unsafe fn property_key_utf8(
    env: Env,
    data: *const u8,
    len: i32,
    out: *mut Local,
) -> Result<(), napi::Status> {
    napi::create_property_key_utf8(env, data as *const _, len as usize, out)
}

#[cfg(not(feature = "napi-10"))]
pub(super) unsafe fn property_key_utf8(
    env: Env,
    data: *const u8,
    len: i32,
    out: *mut Local,
) -> Result<(), napi::Status> {
    napi::create_string_utf8(env, data as *const _, len as usize, out)
}

pub unsafe fn utf8_len(env: Env, value: Local) -> usize {
    let mut len = MaybeUninit::uninit();
    napi::get_value_string_utf8(env, value, ptr::null_mut(), 0, len.as_mut_ptr()).unwrap();

    len.assume_init()
}

pub unsafe fn data(env: Env, out: *mut u8, len: usize, value: Local) -> usize {
    let mut read = MaybeUninit::uninit();

    napi::get_value_string_utf8(env, value, out as *mut _, len, read.as_mut_ptr()).unwrap();

    read.assume_init()
}

pub unsafe fn utf16_len(env: Env, value: Local) -> usize {
    let mut len = MaybeUninit::uninit();

    napi::get_value_string_utf16(env, value, ptr::null_mut(), 0, len.as_mut_ptr()).unwrap();

    len.assume_init()
}

pub unsafe fn data_utf16(env: Env, out: *mut u16, len: usize, value: Local) -> usize {
    let mut read = MaybeUninit::uninit();
    napi::get_value_string_utf16(env, value, out, len, read.as_mut_ptr()).unwrap();

    read.assume_init()
}

pub unsafe fn run_script(out: &mut Local, env: Env, value: Local) -> bool {
    let status = napi::run_script(env, value, out as *mut _);

    status == Ok(())
}
