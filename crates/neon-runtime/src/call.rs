//! Facilities for working with `v8::FunctionCallbackInfo` and getting the current `v8::Isolate`.

use std::os::raw::c_void;
use std::ptr::null_mut;
use raw::{FunctionCallbackInfo, Isolate, Persistent};

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
    pub fn set_return(info: &FunctionCallbackInfo, value: &Persistent);

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
    pub fn this(info: &FunctionCallbackInfo, out: &Persistent, isolate: *mut Isolate);

    /// Initializes the `out` argument provided to refer to the value of the
    /// `v8::FunctionCallbackInfo` `Data`.
    #[link_name = "Neon_Call_Data"]
    pub fn data(info: &FunctionCallbackInfo, out: &Persistent, isolate: *mut Isolate);

    /// Gets the number of arguments passed to the function.
    #[link_name = "Neon_Call_Length"]
    pub fn len(info: &FunctionCallbackInfo) -> i32;

    /// Initializes the `out` argument provided to refer to the `i`th argument passed to the function.
    #[link_name = "Neon_Call_Get"]
    pub fn get(info: &FunctionCallbackInfo, isolate: *mut Isolate, i: i32, out: &Persistent);

}
