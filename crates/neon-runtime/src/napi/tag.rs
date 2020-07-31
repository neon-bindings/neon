use raw::{Env, Local};

use nodejs_sys as napi;

/// Return true if an `napi_value` `val` has the expected value type.
unsafe fn is_type(env: Env, val: Local, expect: napi::napi_valuetype) -> bool {
    let mut actual = napi::napi_valuetype::napi_undefined;
    assert_eq!(napi::napi_typeof(env, val, &mut actual as *mut _), napi::napi_status::napi_ok);
    actual == expect
}

pub unsafe extern "C" fn is_undefined(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_undefined)
}

pub unsafe extern "C" fn is_null(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_null)
}

/// Is `val` a JavaScript number?
pub unsafe extern "C" fn is_number(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_number)
}

/// Is `val` a JavaScript boolean?
pub unsafe extern "C" fn is_boolean(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_boolean)
}

/// Is `val` a JavaScript string?
pub unsafe extern "C" fn is_string(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_string)
}

pub unsafe extern "C" fn is_object(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_object)
}

pub unsafe extern "C" fn is_array(env: Env, val: Local) -> bool {
    let mut result = false;
    assert_eq!(napi::napi_is_array(env, val, &mut result as *mut _), napi::napi_status::napi_ok);
    result
}

pub unsafe extern "C" fn is_function(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_function)
}

pub unsafe extern "C" fn is_error(env: Env, val: Local) -> bool {
    let mut result = false;
    assert_eq!(napi::napi_is_error(env, val, &mut result as *mut _), napi::napi_status::napi_ok);
    result
}

/// Is `val` a Node.js Buffer instance?
pub unsafe extern "C" fn is_buffer(env: Env, val: Local) -> bool {
    let mut result = false;
    assert_eq!(napi::napi_is_buffer(env, val, &mut result as *mut _), napi::napi_status::napi_ok);
    result
}

/// Is `val` an ArrayBuffer instance?
pub unsafe extern "C" fn is_arraybuffer(env: Env, val: Local) -> bool {
    let mut result = false;
    assert_eq!(napi::napi_is_arraybuffer(env, val, &mut result as *mut _), napi::napi_status::napi_ok);
    result
}
