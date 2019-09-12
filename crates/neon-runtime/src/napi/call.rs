//! Facilities for working with `v8::FunctionCallbackInfo` and getting the current `v8::Isolate`.

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

// FIXME(napi): #[link_name = "Neon_Call_SetReturn"]
pub extern "C" fn set_return(info: &FunctionCallbackInfo, value: Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Call_GetIsolate"]
pub extern "C" fn get_isolate(info: &FunctionCallbackInfo) -> *mut Isolate { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Call_CurrentIsolate"]
pub extern "C" fn current_isolate() -> *mut Isolate { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Call_IsConstruct"]
pub extern "C" fn is_construct(info: &FunctionCallbackInfo) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Call_This"]
pub extern "C" fn this(info: &FunctionCallbackInfo, out: &mut Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Call_Data"]
pub extern "C" fn data(info: &FunctionCallbackInfo, out: &mut Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Call_Length"]
pub extern "C" fn len(info: &FunctionCallbackInfo) -> i32 { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Call_Get"]
pub extern "C" fn get(info: &FunctionCallbackInfo, i: i32, out: &mut Local) { unimplemented!() }
