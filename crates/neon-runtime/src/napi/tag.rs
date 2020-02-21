use raw::{Env, Local};

use nodejs_sys as napi;

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

pub unsafe extern "C" fn is_number(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_number)
}

pub unsafe extern "C" fn is_boolean(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_boolean)
}

pub unsafe extern "C" fn is_string(env: Env, val: Local) -> bool {
    is_type(env, val, napi::napi_valuetype::napi_string)
}

pub unsafe extern "C" fn is_object(_env: Env, _val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_array(_env: Env, _val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_function(_env: Env, _val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_error(_env: Env, _val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_buffer(_env: Env, _obj: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_arraybuffer(_env: Env, _obj: Local) -> bool { unimplemented!() }
