use nodejs_sys as napi;
use raw::{Env, Local};

pub unsafe extern "C" fn new_date(env: Env, out: *mut Local, value: f64) {
    let status = napi::napi_create_date(env, value, out);
    assert_eq!(status, napi::napi_status::napi_ok);
}

pub unsafe extern "C" fn value(env: Env, p: Local) -> f64 {
    let mut value = 0.0;
    let status = napi::napi_get_date_value(env, p, &mut value as *mut f64);
    assert_eq!(status, napi::napi_status::napi_ok);
    return value;
}
