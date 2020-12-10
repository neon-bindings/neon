use std::mem::MaybeUninit;

use crate::napi::bindings as napi;
use crate::raw::{Env, Local};

pub unsafe fn new(env: Env) -> (napi::Deferred, Local) {
    let mut deferred = MaybeUninit::uninit();
    let mut promise = MaybeUninit::uninit();

    assert_eq!(
        napi::create_promise(env, deferred.as_mut_ptr(), promise.as_mut_ptr()),
        napi::Status::Ok,
    );

    (deferred.assume_init(), promise.assume_init())
}

pub unsafe fn resolve(env: Env, deferred: napi::Deferred, value: Local) {
    assert_eq!(napi::resolve_deferred(env, deferred, value), napi::Status::Ok);
}

pub unsafe fn reject(env: Env, deferred: napi::Deferred, value: Local) {
    assert_eq!(napi::reject_deferred(env, deferred, value), napi::Status::Ok);
}
