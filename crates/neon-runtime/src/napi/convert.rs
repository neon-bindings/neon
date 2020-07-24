use nodejs_sys as napi;

use raw::{Env, Local};

pub unsafe extern "C" fn to_object(out: &mut Local, env: Env, value: Local) -> bool {
    let status = napi::napi_coerce_to_object(env, value, out as *mut _);

    status == napi::napi_status::napi_ok
}

pub unsafe extern "C" fn to_string(out: &mut Local, env: Env, value: Local) -> bool {
    let status = napi::napi_coerce_to_string(env, value, out as *mut _);

    status == napi::napi_status::napi_ok
}
