use std::os::raw::c_void;
use raw::{FunctionCallbackInfo, Isolate, Local};

extern "system" {

    #[link_name = "NeonSys_Class_ForConstructor"]
    pub fn for_constructor(info: &FunctionCallbackInfo, out: &mut Local);

    #[link_name = "NeonSys_Class_ForMethod"]
    pub fn for_method(info: &FunctionCallbackInfo, out: &mut Local);

    #[link_name = "NeonSys_Class_Create"]
    pub fn create(out: &mut Local, isolate: *mut Isolate, callback: *mut c_void, construct_kernel: *mut c_void) -> bool;

    #[link_name = "NeonSys_Class_GetConstructorKernel"]
    pub fn get_constructor_kernel(obj: Local) -> *mut c_void;

    #[link_name = "NeonSys_Class_ExecConstructorKernel"]
    pub fn exec_constructor_kernel(closure: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void), info: &FunctionCallbackInfo, scope: *mut c_void);

    #[link_name = "NeonSys_Class_Constructor"]
    pub fn constructor(out: &mut Local, ft: Local);

    #[link_name = "NeonSys_Class_Check"]
    pub fn check(c: Local, v: Local) -> bool;

    #[link_name = "NeonSys_Class_GetInstanceInternals"]
    pub fn get_instance_internals(obj: Local) -> *mut c_void;

/*
    #[link_name = "NeonSys_Class_SetCallHandler"]
    pub fn set_call_handler(ft: Local, isolate: *mut Isolate, callback: *mut c_void, construct_kernel: *mut c_void);
*/

}
