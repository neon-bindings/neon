use std::mem::MaybeUninit;

use nodejs_sys as napi;

use raw::{Local, Env};

pub unsafe fn new(env: Env, value: Local) -> napi::napi_ref {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_create_reference(env, value, 1, result.as_mut_ptr()),
        napi::napi_status::napi_ok,
    );

    result.assume_init()
}

pub unsafe fn reference(env: Env, value: napi::napi_ref) -> usize {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_reference_ref(env, value, result.as_mut_ptr()),
        napi::napi_status::napi_ok,
    );

    result.assume_init() as usize
}

pub unsafe fn unreference(env: Env, value: napi::napi_ref) -> usize {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_reference_unref(env, value, result.as_mut_ptr()),
        napi::napi_status::napi_ok,
    );

    result.assume_init() as usize
}

pub unsafe fn get(env: Env, value: napi::napi_ref) -> Local {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_get_reference_value(env, value, result.as_mut_ptr()),
        napi::napi_status::napi_ok,
    );

    result.assume_init()
}
