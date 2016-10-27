use std::os::raw::c_void;
use raw::{FunctionCallbackInfo, Local};

extern "C" {

    #[link_name = "NeonSys_Fun_New"]
    pub fn new(out: &mut Local, isolate: *mut c_void, callback: *mut c_void, kernel: *mut c_void) -> bool;

    #[link_name = "NeonSys_Fun_New_Template"]
    pub fn new_template(out: &mut Local, isolate: *mut c_void, callback: *mut c_void, kernel: *mut c_void) -> bool;

    #[link_name = "NeonSys_Fun_ExecKernel"]
    pub fn exec_kernel(kernel: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void), info: &FunctionCallbackInfo, scope: *mut c_void);

    #[link_name = "NeonSys_Fun_GetKernel"]
    pub fn get_kernel(obj: Local) -> *mut c_void;

    #[link_name = "NeonSys_Fun_Call"]
    pub fn call(out: &mut Local, isolate: *mut c_void, fun: Local, this: Local, argc: i32, argv: *mut c_void) -> bool;

    #[link_name = "NeonSys_Fun_Construct"]
    pub fn construct(out: &mut Local, isolate: *mut c_void, fun: Local, argc: i32, argv: *mut c_void) -> bool;

}
