use std::mem::MaybeUninit;

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn new(env: Env, value: Local) -> napi::Ref {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::create_reference(env, value, 1, result.as_mut_ptr()),
        napi::Status::Ok,
    );

    result.assume_init()
}

/// # Safety
/// Must only be used from the same module context that created the reference
pub unsafe fn reference(env: Env, value: napi::Ref) -> usize {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::reference_ref(env, value, result.as_mut_ptr()),
        napi::Status::Ok,
    );

    result.assume_init() as usize
}

/// # Safety
/// Must only be used from the same module context that created the reference
pub unsafe fn unreference(env: Env, value: napi::Ref) {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::reference_unref(env, value, result.as_mut_ptr()),
        napi::Status::Ok,
    );

    if result.assume_init() == 0 {
        assert_eq!(napi::delete_reference(env, value), napi::Status::Ok);
    }
}

/// # Safety
/// Must only be used from the same module context that created the reference
pub unsafe fn get(env: Env, value: napi::Ref) -> Local {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::get_reference_value(env, value, result.as_mut_ptr()),
        napi::Status::Ok,
    );

    result.assume_init()
}
