//! Facilities for working with Array `napi_value`s.

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn new(out: &mut Local, env: Env, length: u32) {
    assert_eq!(
        napi::create_array_with_length(env, length as usize, out as *mut _),
        napi::Status::Ok,
    );
}

/// Gets the length of a `napi_value` containing a JavaScript Array.
///
/// # Panics
/// This function panics if `array` is not an Array, or if a previous n-api call caused a pending
/// exception.
pub unsafe fn len(env: Env, array: Local) -> u32 {
    let mut len = 0;
    assert_eq!(
        napi::get_array_length(env, array, &mut len as *mut _),
        napi::Status::Ok
    );
    len
}
