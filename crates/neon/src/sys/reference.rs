use std::mem::MaybeUninit;

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn new(env: Env, value: Local) -> napi::Ref {
    let mut result = MaybeUninit::uninit();

    unsafe {
        napi::create_reference(env, value, 1, result.as_mut_ptr()).unwrap();
        result.assume_init()
    }
}

/// # Safety
/// Must only be used from the same module context that created the reference
pub unsafe fn reference(env: Env, value: napi::Ref) -> usize {
    let mut result = MaybeUninit::uninit();

    unsafe {
        napi::reference_ref(env, value, result.as_mut_ptr()).unwrap();
        result.assume_init() as usize
    }
}

/// # Safety
/// Must only be used from the same module context that created the reference
pub unsafe fn unreference(env: Env, value: napi::Ref) {
    let mut result = MaybeUninit::uninit();

    unsafe {
        napi::reference_unref(env, value, result.as_mut_ptr()).unwrap();

        if result.assume_init() == 0 {
            napi::delete_reference(env, value).unwrap();
        }
    }
}

/// # Safety
/// Must only be used from the same module context that created the reference
pub unsafe fn get(env: Env, value: napi::Ref) -> Local {
    let mut result = MaybeUninit::uninit();

    unsafe {
        napi::get_reference_value(env, value, result.as_mut_ptr()).unwrap();
        result.assume_init()
    }
}
