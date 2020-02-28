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

pub unsafe extern "C" fn this(_info: FunctionCallbackInfo, _out: &mut Local) { unimplemented!() }

pub unsafe extern "C" fn data(env: Env, info: FunctionCallbackInfo, out: &mut *mut c_void) {
    let mut data = null_mut();
    let mut argc = 0usize;
    let status = napi::napi_get_cb_info(
        env,
        info,
        &mut argc as *mut _,
        null_mut(),
        null_mut(),
        &mut data as *mut _,
    );
    println!("data() status = {:?} argc = {}", status, argc);
    if status == napi::napi_status::napi_ok {
        *out = data;
    }
}

pub unsafe extern "C" fn len(_info: FunctionCallbackInfo) -> i32 { unimplemented!() }

pub unsafe extern "C" fn get(_info: FunctionCallbackInfo, _i: i32, _out: &mut Local) { unimplemented!() }
