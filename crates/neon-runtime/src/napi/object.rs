use std::mem::MaybeUninit;

use crate::napi::bindings as napi;
use crate::raw::{Env, Local};

/// Mutates the `out` argument to refer to a `napi_value` containing a newly created JavaScript Object.
pub unsafe extern "C" fn new(out: &mut Local, env: Env) {
    napi::create_object(env, out as *mut _);
}

#[cfg(feature = "napi-6")]
/// Mutates the `out` argument to refer to a `napi_value` containing the own property names of the
/// `object` as a JavaScript Array.
pub unsafe extern "C" fn get_own_property_names(out: &mut Local, env: Env, object: Local) -> bool {
    let mut property_names = MaybeUninit::uninit();

    if napi::get_all_property_names(
        env,
        object,
        napi::KeyCollectionMode::OwnOnly,
        napi::KeyFilter::ALL_PROPERTIES | napi::KeyFilter::SKIP_SYMBOLS,
        napi::KeyConversion::NumbersToStrings,
        property_names.as_mut_ptr(),
    ) != napi::Status::Ok {
        return false;
    }

    *out = property_names.assume_init();

    true
}

// Unused.
pub unsafe extern "C" fn get_isolate(_obj: Local) -> Env {
    unimplemented!()
}

/// Mutate the `out` argument to refer to the value at `index` in the given `object`. Returns `false` if the value couldn't be retrieved.
pub unsafe extern "C" fn get_index(out: &mut Local, env: Env, object: Local, index: u32) -> bool {
    let status = napi::get_element(env, object, index, out as *mut _);

    status == napi::Status::Ok
}

/// Sets the key value of a `napi_value` at the `index` provided. Returns `true` if the set
/// succeeded.
///
/// The `out` parameter and the return value contain the same information for historical reasons,
/// see [discussion].
///
/// [discussion]: https://github.com/neon-bindings/neon/pull/458#discussion_r344827965
pub unsafe extern "C" fn set_index(out: &mut bool, env: Env, object: Local, index: u32, val: Local) -> bool {
    let status = napi::set_element(env, object, index, val);
    *out = status == napi::Status::Ok;

    *out
}

/// Mutate the `out` argument to refer to the value at a named `key` in the given `object`. Returns `false` if the value couldn't be retrieved.
pub unsafe extern "C" fn get_string(env: Env, out: &mut Local, object: Local, key: *const u8, len: i32) -> bool {
    let mut key_val = MaybeUninit::uninit();

    // Not using `crate::string::new()` because it requires a _reference_ to a Local,
    // while we only have uninitialized memory.
    if napi::create_string_utf8(env, key as *const i8, len as usize, key_val.as_mut_ptr())
        != napi::Status::Ok
    {
        return false;
    }

    // Not using napi_get_named_property() because the `key` may not be null terminated.
    if napi::get_property(env, object, key_val.assume_init(), out as *mut _)
        != napi::Status::Ok
    {
        return false;
    }

    true
}

/// Sets the key value of a `napi_value` at a named key. Returns `true` if the set succeeded.
///
/// The `out` parameter and the return value contain the same information for historical reasons,
/// see [discussion].
///
/// [discussion]: https://github.com/neon-bindings/neon/pull/458#discussion_r344827965
pub unsafe extern "C" fn set_string(env: Env, out: &mut bool, object: Local, key: *const u8, len: i32, val: Local) -> bool {
    let mut key_val = MaybeUninit::uninit();

    *out = true;

    if napi::create_string_utf8(env, key as *const i8, len as usize, key_val.as_mut_ptr())
        != napi::Status::Ok
    {
        *out = false;
        return false;
    }

    if napi::set_property(env, object, key_val.assume_init(), val)
        != napi::Status::Ok
    {
        *out = false;
        return false;
    }

    true
}

/// Mutates `out` to refer to the value of the property of `object` named by the `key` value.
/// Returns false if the value couldn't be retrieved.
pub unsafe extern "C" fn get(out: &mut Local, env: Env, object: Local, key: Local) -> bool {
    let status = napi::get_property(env, object, key, out as *mut _);

    status == napi::Status::Ok
}

/// Sets the property value of an `napi_value` object, named by another `value` `key`. Returns `true` if the set succeeded.
///
/// The `out` parameter and the return value contain the same information for historical reasons,
/// see [discussion].
///
/// [discussion]: https://github.com/neon-bindings/neon/pull/458#discussion_r344827965
pub unsafe extern "C" fn set(out: &mut bool, env: Env, object: Local, key: Local, val: Local) -> bool {
    let status = napi::set_property(env, object, key, val);
    *out = status == napi::Status::Ok;

    *out
}
