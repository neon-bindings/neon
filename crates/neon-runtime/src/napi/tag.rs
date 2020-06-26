use raw::{Env, Local};

use nodejs_sys as napi;

/// Return true if an `napi_value` `val` has the expected value type.
unsafe fn is_type(env: Env, val: Local, expect: napi::napi_valuetype) -> bool {
    let mut actual = napi::napi_valuetype::napi_undefined;
    if napi::napi_typeof(env, val, &mut actual as *mut _) == napi::napi_status::napi_ok {
        actual == expect
    } else {
        false
    }
}

pub unsafe extern "C" fn is_undefined(_env: Env, _val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_null(_env: Env, _val: Local) -> bool { unimplemented!() }

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

pub unsafe extern "C" fn is_array(_env: Env, _val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_function(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_function)
}

pub unsafe extern "C" fn is_error(_env: Env, _val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_buffer(_env: Env, _obj: Local) -> bool { unimplemented!() }

/// Is `val` an ArrayBuffer instance?
pub unsafe extern "C" fn is_arraybuffer(env: Env, val: Local) -> bool {
    let mut result = false;
    assert_eq!(napi::napi_is_arraybuffer(env, val, &mut result as *mut _), napi::napi_status::napi_ok);
    result
}
