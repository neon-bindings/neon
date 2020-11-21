use crate::raw::{Env, Local};
use crate::napi::bindings as napi;

/// Return true if an `napi_value` `val` has the expected value type.
unsafe fn is_type(env: Env, val: Local, expect: napi::ValueType) -> bool {
    let mut actual = napi::ValueType::Undefined;
    assert_eq!(napi::typeof_value(env, val, &mut actual as *mut _), napi::Status::Ok);
    actual == expect
}

pub unsafe extern "C" fn is_undefined(env: Env, val: Local) -> bool {
    is_type(env, val, napi::ValueType::Undefined)
}

pub unsafe extern "C" fn is_null(env: Env, val: Local) -> bool {
    is_type(env, val, napi::ValueType::Null)
}

/// Is `val` a JavaScript number?
pub unsafe extern "C" fn is_number(env: Env, val: Local) -> bool {
    is_type(env, val, napi::ValueType::Number)
}

/// Is `val` a JavaScript boolean?
pub unsafe extern "C" fn is_boolean(env: Env, val: Local) -> bool {
    is_type(env, val, napi::ValueType::Boolean)
}

/// Is `val` a JavaScript string?
pub unsafe extern "C" fn is_string(env: Env, val: Local) -> bool {
    is_type(env, val, napi::ValueType::String)
}

pub unsafe extern "C" fn is_object(env: Env, val: Local) -> bool {
    is_type(env, val, napi::ValueType::Object)
}

pub unsafe extern "C" fn is_array(env: Env, val: Local) -> bool {
    let mut result = false;
    assert_eq!(napi::is_array(env, val, &mut result as *mut _), napi::Status::Ok);
    result
}

pub unsafe extern "C" fn is_function(env: Env, val: Local) -> bool {
    is_type(env, val, napi::ValueType::Function)
}

pub unsafe extern "C" fn is_error(env: Env, val: Local) -> bool {
    let mut result = false;
    assert_eq!(napi::is_error(env, val, &mut result as *mut _), napi::Status::Ok);
    result
}

/// Is `val` a Node.js Buffer instance?
pub unsafe extern "C" fn is_buffer(env: Env, val: Local) -> bool {
    let mut result = false;
    assert_eq!(napi::is_buffer(env, val, &mut result as *mut _), napi::Status::Ok);
    result
}

/// Is `val` an ArrayBuffer instance?
pub unsafe extern "C" fn is_arraybuffer(env: Env, val: Local) -> bool {
    let mut result = false;
    assert_eq!(napi::is_arraybuffer(env, val, &mut result as *mut _), napi::Status::Ok);
    result
}

pub unsafe extern "C" fn is_date(env: Env, val: Local) -> bool {
    let mut result = false;
    assert_eq!(napi::napi_is_date(env, val, &mut result as *mut _), napi::napi_status::napi_ok);
    result
}
