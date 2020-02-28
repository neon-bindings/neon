//! Facilities for working with JS functions.

use call::CCallback;
use raw::{Env, Local};
use std::os::raw::c_void;
use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};

use nodejs_sys as napi;

/// Mutates the `out` argument provided to refer to a newly created `v8::Function`. Returns
/// `false` if the value couldn't be created.
pub unsafe extern "C" fn new(out: &mut Local, env: Env, callback: CCallback) -> bool {
    let status = napi::napi_create_function(
        env,
        null(),
        0,
        Some(std::mem::transmute(callback.static_callback)),
        callback.dynamic_callback,
        out as *mut Local,
    );

    status == napi::napi_status::napi_ok
}

pub unsafe extern "C" fn new_template(_out: &mut Local, _env: Env, _callback: CCallback) -> bool {
    unimplemented!()
}

pub unsafe extern "C" fn get_dynamic_callback(env: Env, data: *mut c_void) -> *mut c_void {
    data
    /*
    let mut value = null_mut();

    // debug stuff
    {
        let mut ty = napi::napi_valuetype::napi_null;
        let r = napi::napi_typeof(env, obj, &mut ty as *mut _);
        dbg!(r,ty);
        let d = &mut [0u8; 64];
        let l = crate::string::utf8_len(env, obj);
        crate::string::data(env, d.as_mut_ptr(), l, obj);
        dbg!(std::str::from_utf8(&d[..l as usize]));
    }
    // end debug stuff

    let status = napi::napi_get_value_external(
        env,
        obj,
        &mut value as *mut _,
    );
    println!("get_dynamic_callback() status = {:?}", status);
    if status != napi::napi_status::napi_ok {
        return null_mut();
    }
    value
    */
}

pub unsafe extern "C" fn call(
    _out: &mut Local,
    _env: Env,
    _fun: Local,
    _this: Local,
    _argc: i32,
    _argv: *mut c_void,
) -> bool {
    unimplemented!()
}

pub unsafe extern "C" fn construct(
    _out: &mut Local,
    _env: Env,
    _fun: Local,
    _argc: i32,
    _argv: *mut c_void,
) -> bool {
    unimplemented!()
}
