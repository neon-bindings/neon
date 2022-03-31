use std::mem::MaybeUninit;

use super::{
    bindings as napi,
    raw::{Env, Local},
};

/// Create a new date object
///
/// # Safety
///
/// `env` is a raw pointer. Please ensure it points to a napi_env that is valid for the current context.
pub unsafe fn new_date(env: Env, value: f64) -> Local {
    let mut local = MaybeUninit::zeroed();
    let status = napi::create_date(env, value, local.as_mut_ptr());
    assert_eq!(status, napi::Status::Ok);
    local.assume_init()
}

/// Get the value of a date object
///
/// # Safety
///
/// `env` is a raw pointer. Please ensure it points to a napi_env that is valid for the current context.
/// `Local` must be an NAPI value associated with the given `Env`
pub unsafe fn value(env: Env, p: Local) -> f64 {
    let mut value = 0.0;
    let status = napi::get_date_value(env, p, &mut value as *mut _);
    assert_eq!(status, napi::Status::Ok);
    value
}
