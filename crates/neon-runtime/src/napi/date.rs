use std::mem::MaybeUninit;
use crate::napi::bindings as napi;
use crate::raw::{Env, Local};

pub unsafe fn new_date(env: Env, value: f64) -> Local {
    let mut local = MaybeUninit::zeroed();
    let status = napi::create_date(env, value, local.as_mut_ptr());
    assert_eq!(status, napi::Status::Ok);
    local.assume_init()
}

pub unsafe fn value(env: Env, p: Local) -> f64 {
    let mut value = 0.0;
    let status = napi::get_date_value(env, p, &mut value as *mut _);
    assert_eq!(status, napi::Status::Ok);
    return value;
}
