//! Facilities for working with `v8::Function`s.

use std::os::raw::c_void;
use raw::{FunctionCallbackInfo, Local};

extern "system" {

    /// Mutates the `out` argument provided to refer to a newly created `v8::Function`. Returns
    /// `false` if the value couldn't be created.
    #[link_name = "NeonSys_Fun_New"]
    pub fn new(out: &mut Local, isolate: *mut c_void, callback: *mut c_void, kernel: *mut c_void) -> bool;

    /// Mutates the `out` argument provided to refer to a newly created `v8::FunctionTemplate`.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "NeonSys_Fun_Template_New"]
    pub fn new_template(out: &mut Local, isolate: *mut c_void, callback: *mut c_void, kernel: *mut c_void) -> bool;

    /// Creates a new `v8::HandleScope` and calls the `callback` provided with the the argument
    /// signature `(info, kernel, scope)`.
    #[link_name = "NeonSys_Fun_ExecKernel"]
    pub fn exec_kernel(kernel: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void), info: &FunctionCallbackInfo, scope: *mut c_void);

    /// Gets the reference to the `v8::Local<v8::External>` handle provided.
    #[link_name = "NeonSys_Fun_GetKernel"]
    pub fn get_kernel(obj: Local) -> *mut c_void;

    /// Calls the function provided (`fun`) and mutates the `out` argument provided to refer to the
    /// result of the function call. Returns `false` if the result of the call was empty.
    #[link_name = "NeonSys_Fun_Call"]
    pub fn call(out: &mut Local, isolate: *mut c_void, fun: Local, this: Local, argc: i32, argv: *mut c_void) -> bool;

    /// Makes a constructor call to with the function provided (`fun`) and mutates the `out`
    /// argument provided to refer to the result of the constructor call. Returns `false` if the
    /// result of the call was empty.
    #[link_name = "NeonSys_Fun_Construct"]
    pub fn construct(out: &mut Local, isolate: *mut c_void, fun: Local, argc: i32, argv: *mut c_void) -> bool;

}
