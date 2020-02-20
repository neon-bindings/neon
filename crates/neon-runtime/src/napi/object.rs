use std::mem::MaybeUninit;

use nodejs_sys as napi;

use raw::{Env, Local};

pub unsafe extern "C" fn new(out: &mut Local, env: Env) {
    napi::napi_create_object(env, out as *mut Local);
}

pub unsafe extern "C" fn get_own_property_names(_out: &mut Local, _object: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn get_isolate(_obj: Local) -> Env { unimplemented!() }

pub unsafe extern "C" fn get_index(_out: &mut Local, _object: Local, _index: u32) -> bool { unimplemented!() }

pub unsafe extern "C" fn set_index(_out: &mut bool, _object: Local, _index: u32, _val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn get_string(env: Env, out: &mut Local, object: Local, key: *const u8, len: i32) -> bool {
    let mut key_val = MaybeUninit::uninit();

    // Not using `crate::string::new()` because it requires a _reference_ to a Local,
    // while we only have uninitialized memory.
    if napi::napi_create_string_utf8(
        env,
        key as *const i8,
        len as usize,
        key_val.as_mut_ptr(),
    ) != napi::napi_status::napi_ok {
        return false;
    }

    // Not using napi_get_named_property() because the `key` may not be null terminated.
    if napi::napi_get_property(
        env,
        object,
        key_val.assume_init(),
        out as *mut Local,
    ) != napi::napi_status::napi_ok {
        return false;
    }

    true
}

pub unsafe extern "C" fn set_string(
    env: Env,
    out: &mut bool,
    object: Local,
    key: *const u8,
    len: i32,
    val: Local,
) -> bool {
    let mut key_val = MaybeUninit::uninit();

    *out = true;

    if napi::napi_create_string_utf8(
        env,
        key as *const i8,
        len as usize,
        key_val.as_mut_ptr(),
    ) != napi::napi_status::napi_ok {
        *out = false;
        return false;
    }

    if napi::napi_set_property(
        env,
        object,
        key_val.assume_init(),
        val,
    ) != napi::napi_status::napi_ok {
        *out = false;
        return false;
    }

    true
}

pub unsafe extern "C" fn get(out: &mut Local, env: Env, object: Local, key: Local) -> bool {
    let status = napi::napi_get_property(env, object, key, out as *mut Local);

    status == napi::napi_status::napi_ok
}

pub unsafe extern "C" fn set(out: &mut bool, env: Env, object: Local, key: Local, val: Local) -> bool {
    let status = napi::napi_set_property(env, object, key, val);
    *out = status == napi::napi_status::napi_ok;

    *out
}
