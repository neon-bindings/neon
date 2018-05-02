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

extern "C" {

    /// Sets the return value of the function call.
    #[link_name = "Neon_Call_SetReturn"]
    pub fn set_return(info: &FunctionCallbackInfo, value: Local);

    /// Gets the isolate of the function call.
    #[link_name = "Neon_Call_GetIsolate"]
    pub fn get_isolate(info: &FunctionCallbackInfo) -> *mut Isolate;

    /// Gets the current `v8::Isolate`.
    #[link_name = "Neon_Call_CurrentIsolate"]
    pub fn current_isolate() -> *mut Isolate;

    /// Indicates if the function call was invoked as a constructor.
    #[link_name = "Neon_Call_IsConstruct"]
    pub fn is_construct(info: &FunctionCallbackInfo) -> bool;

    /// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the object
    /// the function is bound to.
    #[link_name = "Neon_Call_This"]
    pub fn this(info: &FunctionCallbackInfo, out: &mut Local);

    /// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the
    /// currently executing function.
    #[link_name = "Neon_Call_Callee"]
    pub fn callee(info: &FunctionCallbackInfo, out: &mut Local) -> bool;

    /// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the
    /// `v8::FunctionCallbackInfo` `Data`.
    #[link_name = "Neon_Call_Data"]
    pub fn data(info: &FunctionCallbackInfo, out: &mut Local);

    /// Gets the number of arguments passed to the function.
    #[link_name = "Neon_Call_Length"]
    pub fn len(info: &FunctionCallbackInfo) -> i32;

    /// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the `i`th
    /// argument passed to the function.
    #[link_name = "Neon_Call_Get"]
    pub fn get(info: &FunctionCallbackInfo, i: i32, out: &mut Local);

}
