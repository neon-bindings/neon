//! Facilities for working with Array `napi_value`s.

use raw::{Local, Env};

use nodejs_sys as napi;

pub unsafe extern "C" fn new(_out: &mut Local, _env: Env, _length: u32) { unimplemented!() }

/// Gets the length of a `napi_value` containing a JavaScript Array.
pub unsafe extern "C" fn len(env: Env, array: Local) -> u32 {
    let mut len = 0;
    assert_eq!(napi::napi_get_array_length(env, array, &mut len as *mut _), napi::napi_status::napi_ok);
    len
}
