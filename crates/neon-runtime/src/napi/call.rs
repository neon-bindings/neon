use std::os::raw::c_void;
use std::ptr::null_mut;
use raw::{FunctionCallbackInfo, Env, Local};

use nodejs_sys as napi;

#[repr(C)]
pub struct CCallback {
    pub static_callback: *mut c_void,
    pub dynamic_callback: *mut c_void
}

impl Default for CCallback {
    fn default() -> Self {
        CCallback {
            static_callback: null_mut(),
            dynamic_callback: null_mut()
        }
    }
}

pub unsafe extern "C" fn set_return(_info: FunctionCallbackInfo, _value: Local) {

}

pub unsafe extern "C" fn get_isolate(info: FunctionCallbackInfo) -> Env { unimplemented!() }

// FIXME: Remove. This will never be implemented
pub unsafe extern "C" fn current_isolate() -> Env { panic!("current_isolate won't be implemented in n-api") }

pub unsafe extern "C" fn is_construct(_info: FunctionCallbackInfo) -> bool { unimplemented!() }

pub unsafe extern "C" fn this(env: Env, info: FunctionCallbackInfo, out: &mut Local) {
    let status = napi::napi_get_cb_info(
        env,
        info,
        null_mut(),
        null_mut(),
        out as *mut _,
        null_mut(),
    );
    assert_eq!(status, napi::napi_status::napi_ok);
}

/// Mutates the `out` argument provided to refer to the associated data value of the
/// `napi_callback_info`.
pub unsafe extern "C" fn data(env: Env, info: FunctionCallbackInfo, out: &mut *mut c_void) {
    let mut data = null_mut();
    let status = napi::napi_get_cb_info(
        env,
        info,
        null_mut(),
        null_mut(),
        null_mut(),
        &mut data as *mut _,
    );
    if status == napi::napi_status::napi_ok {
        *out = data;
    }
}

/// Gets the number of arguments passed to the function.
pub unsafe extern "C" fn len(env: Env, info: FunctionCallbackInfo) -> i32 {
    let mut argc = 0usize;
    let status = napi::napi_get_cb_info(
        env,
        info,
        &mut argc as *mut _,
        null_mut(),
        null_mut(),
        null_mut(),
    );
    assert_eq!(status, napi::napi_status::napi_ok);
    argc as i32
}

/// Mutates the `out` argument provided to refer to the `napi_value` of the `i`th argument
/// passed to the function.
pub unsafe extern "C" fn get(env: Env, info: FunctionCallbackInfo, i: i32, out: &mut Local) {
    // TODO make this not allocate
    // Instead, we can probably get all the arguments at once in `neon` itself?
    let mut args = vec![null_mut(); (i + 1) as usize];
    let mut num_args = args.len();

    let status = napi::napi_get_cb_info(
        env,
        info,
        &mut num_args as *mut _,
        args.as_mut_ptr(),
        null_mut(),
        null_mut(),
    );
    assert_eq!(status, napi::napi_status::napi_ok);
    assert!(num_args > i as usize);
    *out = args[i as usize];
}
