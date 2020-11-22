use std::mem::MaybeUninit;
use nodejs_sys as napi;
use raw::{Env, Local};

pub unsafe extern "C" fn new_date(env: Env, value: f64) -> Local {
    let mut local = MaybeUninit::zeroed();
    let status = napi::napi_create_date(env, value, local.as_mut_ptr());
    assert_eq!(status, napi::napi_status::napi_ok);
    local.assume_init()
}

pub unsafe extern "C" fn value(env: Env, p: Local) -> f64 {
    let mut value = 0.0;
    let status = napi::napi_get_date_value(env, p, &mut value as *mut _);
    assert_eq!(status, napi::napi_status::napi_ok);
    return value;
}
