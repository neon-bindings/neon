use std::mem::MaybeUninit;

use super::{
    bindings as napi,
    raw::{Env, Local},
};

/// Mutates the `out` argument to refer to a `napi_value` containing a newly created JavaScript Object.
pub unsafe fn new(out: &mut Local, env: Env) {
    unsafe { napi::create_object(env, out as *mut _) }.unwrap();
}

#[cfg(feature = "napi-8")]
pub unsafe fn freeze(env: Env, obj: Local) -> Result<(), napi::Status> {
    let status = unsafe { napi::object_freeze(env, obj) };
    debug_assert!(matches!(
        status,
        Ok(()) | Err(napi::Status::PendingException | napi::Status::GenericFailure)
    ));
    status
}

#[cfg(feature = "napi-8")]
pub unsafe fn seal(env: Env, obj: Local) -> Result<(), napi::Status> {
    unsafe { napi::object_seal(env, obj) }
}

#[cfg(feature = "napi-6")]
/// Mutates the `out` argument to refer to a `napi_value` containing the own property names of the
/// `object` as a JavaScript Array.
pub unsafe fn get_own_property_names(out: &mut Local, env: Env, object: Local) -> bool {
    let mut property_names = MaybeUninit::uninit();

    unsafe {
        match napi::get_all_property_names(
            env,
            object,
            napi::KeyCollectionMode::OwnOnly,
            napi::KeyFilter::ALL_PROPERTIES | napi::KeyFilter::SKIP_SYMBOLS,
            napi::KeyConversion::NumbersToStrings,
            property_names.as_mut_ptr(),
        ) {
            Err(napi::Status::PendingException) => return false,
            status => status.unwrap(),
        }

        *out = property_names.assume_init();
    }

    true
}

/// Mutate the `out` argument to refer to the value at `index` in the given `object`. Returns `false` if the value couldn't be retrieved.
pub unsafe fn get_index(out: &mut Local, env: Env, object: Local, index: u32) -> bool {
    let status = unsafe { napi::get_element(env, object, index, out as *mut _) };

    status.is_ok()
}

/// Sets the key value of a `napi_value` at the `index` provided. Returns `true` if the set
/// succeeded.
///
/// The `out` parameter and the return value contain the same information for historical reasons,
/// see [discussion].
///
/// [discussion]: https://github.com/neon-bindings/neon/pull/458#discussion_r344827965
pub unsafe fn set_index(out: &mut bool, env: Env, object: Local, index: u32, val: Local) -> bool {
    let status = unsafe { napi::set_element(env, object, index, val) };
    *out = status.is_ok();

    *out
}

/// Mutate the `out` argument to refer to the value at a named `key` in the given `object`. Returns `false` if the value couldn't be retrieved.
pub unsafe fn get_string(
    env: Env,
    out: &mut Local,
    object: Local,
    key: *const u8,
    len: i32,
) -> bool {
    let mut key_val = MaybeUninit::uninit();

    unsafe {
        // Not using `crate::string::new()` because it requires a _reference_ to a Local,
        // while we only have uninitialized memory.
        match napi::create_string_utf8(env, key as *const _, len as usize, key_val.as_mut_ptr()) {
            Err(napi::Status::PendingException) => return false,
            status => status.unwrap(),
        }

        // Not using napi_get_named_property() because the `key` may not be null terminated.
        match napi::get_property(env, object, key_val.assume_init(), out as *mut _) {
            Err(napi::Status::PendingException) => return false,
            status => status.unwrap(),
        }
    }

    true
}

/// Sets the key value of a `napi_value` at a named key. Returns `true` if the set succeeded.
///
/// The `out` parameter and the return value contain the same information for historical reasons,
/// see [discussion].
///
/// [discussion]: https://github.com/neon-bindings/neon/pull/458#discussion_r344827965
pub unsafe fn set_string(
    env: Env,
    out: &mut bool,
    object: Local,
    key: *const u8,
    len: i32,
    val: Local,
) -> bool {
    let mut key_val = MaybeUninit::uninit();

    unsafe {
        *out = true;

        match napi::create_string_utf8(env, key as *const _, len as usize, key_val.as_mut_ptr()) {
            Err(napi::Status::PendingException) => {
                *out = false;
                return false;
            }
            status => status.unwrap(),
        }

        match napi::set_property(env, object, key_val.assume_init(), val) {
            Err(napi::Status::PendingException) => {
                *out = false;
                return false;
            }
            status => status.unwrap(),
        }
    }

    true
}

/// Mutates `out` to refer to the value of the property of `object` named by the `key` value.
/// Returns false if the value couldn't be retrieved.
pub unsafe fn get(out: &mut Local, env: Env, object: Local, key: Local) -> bool {
    let status = unsafe { napi::get_property(env, object, key, out as *mut _) };

    status.is_ok()
}

/// Sets the property value of an `napi_value` object, named by another `value` `key`. Returns `true` if the set succeeded.
///
/// The `out` parameter and the return value contain the same information for historical reasons,
/// see [discussion].
///
/// [discussion]: https://github.com/neon-bindings/neon/pull/458#discussion_r344827965
pub unsafe fn set(out: &mut bool, env: Env, object: Local, key: Local, val: Local) -> bool {
    let status = unsafe { napi::set_property(env, object, key, val) };
    *out = status.is_ok();

    *out
}
