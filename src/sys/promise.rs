//! JavaScript Promise and Deferred handle
//!
//! https://nodejs.org/api/n-api.html#n_api_promises

use std::mem::MaybeUninit;

use super::{bindings as napi, raw::Env};

/// Create a `Promise` and a `napi::Deferred` handle for resolving it
///
/// # Safety
/// * `env` is a valid `napi_env` for the current thread
/// * The returned `napi::Value` does not outlive `env`
pub unsafe fn create(env: Env) -> (napi::Deferred, napi::Value) {
    let mut deferred = MaybeUninit::uninit();
    let mut promise = MaybeUninit::uninit();

    assert_eq!(
        napi::create_promise(env, deferred.as_mut_ptr(), promise.as_mut_ptr()),
        napi::Status::Ok,
    );

    (deferred.assume_init(), promise.assume_init())
}

/// Resolve a promise from a `napi::Deferred` handle
///
/// # Safety
/// * `env` is a valid `napi_env` for the current thread
/// * `resolution` is a valid `napi::Value`
pub unsafe fn resolve(env: Env, deferred: napi::Deferred, resolution: napi::Value) {
    assert_eq!(
        napi::resolve_deferred(env, deferred, resolution),
        napi::Status::Ok,
    );
}

/// Rejects a promise from a `napi::Deferred` handle
///
/// # Safety
/// * `env` is a valid `napi_env` for the current thread
/// * `rejection` is a valid `napi::Value`
pub unsafe fn reject(env: Env, deferred: napi::Deferred, rejection: napi::Value) {
    assert_eq!(
        napi::reject_deferred(env, deferred, rejection),
        napi::Status::Ok,
    );
}

#[cfg(feature = "napi-6")]
/// Rejects a promise from a `napi::Deferred` handle with a string message
///
/// # Safety
/// * `env` is a valid `napi_env` for the current thread
pub unsafe fn reject_err_message(env: Env, deferred: napi::Deferred, msg: impl AsRef<str>) {
    let msg = super::string(env, msg);
    let mut err = MaybeUninit::uninit();

    assert_eq!(
        napi::create_error(env, std::ptr::null_mut(), msg, err.as_mut_ptr()),
        napi::Status::Ok,
    );

    reject(env, deferred, err.assume_init());
}
