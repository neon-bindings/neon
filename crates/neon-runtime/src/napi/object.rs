use std::mem::MaybeUninit;

use nodejs_sys as napi;

use raw::{Env, Local};

pub unsafe extern "C" fn new(_out: &mut Local) { unimplemented!() }

pub unsafe extern "C" fn get_own_property_names(_out: &mut Local, _object: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn get_isolate(_obj: Local) -> Env { unimplemented!() }

pub unsafe extern "C" fn get_index(_out: &mut Local, _object: Local, _index: u32) -> bool { unimplemented!() }

pub unsafe extern "C" fn set_index(_out: &mut bool, _object: Local, _index: u32, _val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn get_string(_out: &mut Local, _object: Local, _key: *const u8, _len: i32) -> bool { unimplemented!() }

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

pub unsafe extern "C" fn get(_out: &mut Local, _object: Local, _key: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn set(_out: &mut bool, _object: Local, _key: Local, _val: Local) -> bool { unimplemented!() }
