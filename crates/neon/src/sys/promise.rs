//! JavaScript Promise and Deferred handle
//!
//! See: [Promises in Node-API](https://nodejs.org/api/n-api.html#n_api_promises)

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

    let () = napi::create_promise(env, deferred.as_mut_ptr(), promise.as_mut_ptr()).unwrap();

    (deferred.assume_init(), promise.assume_init())
}

/// Resolve a promise from a `napi::Deferred` handle
///
/// # Safety
/// * `env` is a valid `napi_env` for the current thread
/// * `resolution` is a valid `napi::Value`
pub unsafe fn resolve(env: Env, deferred: napi::Deferred, resolution: napi::Value) {
    let () = napi::resolve_deferred(env, deferred, resolution).unwrap();
}

/// Rejects a promise from a `napi::Deferred` handle
///
/// # Safety
/// * `env` is a valid `napi_env` for the current thread
/// * `rejection` is a valid `napi::Value`
pub unsafe fn reject(env: Env, deferred: napi::Deferred, rejection: napi::Value) {
    let () = napi::reject_deferred(env, deferred, rejection).unwrap();
}

#[cfg(feature = "napi-6")]
/// Rejects a promise from a `napi::Deferred` handle with a string message
///
/// # Safety
/// * `env` is a valid `napi_env` for the current thread
pub unsafe fn reject_err_message(env: Env, deferred: napi::Deferred, msg: impl AsRef<str>) {
    let msg = super::string(env, msg);
    let mut err = MaybeUninit::uninit();

    let () = napi::create_error(env, std::ptr::null_mut(), msg, err.as_mut_ptr()).unwrap();

    reject(env, deferred, err.assume_init());
}
