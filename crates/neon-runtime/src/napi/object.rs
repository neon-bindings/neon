use std::mem::MaybeUninit;

use nodejs_sys as napi;

use crate::raw::{Env, Local};

/// Mutates the `out` argument to refer to a `napi_value` containing a newly created JavaScript Object.
pub unsafe extern "C" fn new(out: &mut Local, env: Env) {
    napi::napi_create_object(env, out as *mut _);
}

/// Mutates the `out` argument to refer to a `napi_value` containing the own property names of the
/// `object` as a JavaScript Array.
pub unsafe extern "C" fn get_own_property_names(out: &mut Local, env: Env, object: Local) -> bool {
    let mut property_names = MaybeUninit::uninit();

    if napi::napi_get_all_property_names(
        env,
        object,
        napi::napi_key_collection_mode::napi_key_own_only,
        napi::napi_key_filter::napi_key_all_properties | napi::napi_key_filter::napi_key_skip_symbols,
        napi::napi_key_conversion::napi_key_numbers_to_strings,
        property_names.as_mut_ptr(),
    ) != napi::napi_status::napi_ok {
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
    let status = napi::napi_get_element(env, object, index, out as *mut _);

    status == napi::napi_status::napi_ok
}

/// Sets the key value of a `napi_value` at the `index` provided. Returns `true` if the set
/// succeeded.
///
/// The `out` parameter and the return value contain the same information for historical reasons,
/// see [discussion].
///
/// [discussion]: https://github.com/neon-bindings/neon/pull/458#discussion_r344827965
pub unsafe extern "C" fn set_index(out: &mut bool, env: Env, object: Local, index: u32, val: Local) -> bool {
    let status = napi::napi_set_element(env, object, index, val);
    *out = status == napi::napi_status::napi_ok;

    *out
}

/// Mutate the `out` argument to refer to the value at a named `key` in the given `object`. Returns `false` if the value couldn't be retrieved.
pub unsafe extern "C" fn get_string(env: Env, out: &mut Local, object: Local, key: *const u8, len: i32) -> bool {
    let mut key_val = MaybeUninit::uninit();

    // Not using `crate::string::new()` because it requires a _reference_ to a Local,
    // while we only have uninitialized memory.
    if napi::napi_create_string_utf8(env, key as *const i8, len as usize, key_val.as_mut_ptr())
        != napi::napi_status::napi_ok
    {
        return false;
    }

    // Not using napi_get_named_property() because the `key` may not be null terminated.
    if napi::napi_get_property(env, object, key_val.assume_init(), out as *mut _)
        != napi::napi_status::napi_ok
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

    if napi::napi_create_string_utf8(env, key as *const i8, len as usize, key_val.as_mut_ptr())
        != napi::napi_status::napi_ok
    {
        *out = false;
        return false;
    }

    if napi::napi_set_property(env, object, key_val.assume_init(), val)
        != napi::napi_status::napi_ok
    {
        *out = false;
        return false;
    }

    true
}

/// Mutates `out` to refer to the value of the property of `object` named by the `key` value.
/// Returns false if the value couldn't be retrieved.
pub unsafe extern "C" fn get(out: &mut Local, env: Env, object: Local, key: Local) -> bool {
    let status = napi::napi_get_property(env, object, key, out as *mut _);

    status == napi::napi_status::napi_ok
}

/// Sets the property value of an `napi_value` object, named by another `napi_value` `key`. Returns `true` if the set succeeded.
///
/// The `out` parameter and the return value contain the same information for historical reasons,
/// see [discussion].
///
/// [discussion]: https://github.com/neon-bindings/neon/pull/458#discussion_r344827965
pub unsafe extern "C" fn set(out: &mut bool, env: Env, object: Local, key: Local, val: Local) -> bool {
    let status = napi::napi_set_property(env, object, key, val);
    *out = status == napi::napi_status::napi_ok;

    *out
}
