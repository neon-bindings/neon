use std::os::raw::c_void;
use std::ptr::null_mut;
use raw::{FunctionCallbackInfo, Isolate, Local};

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

pub extern "C" fn set_return(info: &FunctionCallbackInfo, value: Local) { unimplemented!() }

pub extern "C" fn get_isolate(info: &FunctionCallbackInfo) -> *mut Isolate { unimplemented!() }

pub extern "C" fn current_isolate() -> *mut Isolate { unimplemented!() }

pub extern "C" fn is_construct(info: &FunctionCallbackInfo) -> bool { unimplemented!() }

pub extern "C" fn this(info: &FunctionCallbackInfo, out: &mut Local) { unimplemented!() }

pub extern "C" fn data(info: &FunctionCallbackInfo, out: &mut Local) { unimplemented!() }

pub extern "C" fn len(info: &FunctionCallbackInfo) -> i32 { unimplemented!() }

pub extern "C" fn get(info: &FunctionCallbackInfo, i: i32, out: &mut Local) { unimplemented!() }
