//! Facilities for working with `v8::Function`s.

use std::os::raw::c_void;
use call::CCallback;
use raw::Local;

extern "C" {

    /// Mutates the `out` argument provided to refer to a newly created `v8::Function`. Returns
    /// `false` if the value couldn't be created.
    #[link_name = "Neon_Fun_New2"]
    pub fn new_vm2(out: &mut Local, isolate: *mut c_void, callback: CCallback) -> bool;

    /// Mutates the `out` argument provided to refer to a newly created `v8::Function`. Returns
    /// `false` if the value couldn't be created.
    #[link_name = "Neon_Fun_New"]
    pub fn new(out: &mut Local, isolate: *mut c_void, callback: *mut c_void, kernel: *mut c_void) -> bool;

    /// Mutates the `out` argument provided to refer to a newly created `v8::FunctionTemplate`.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "Neon_Fun_Template_New"]
    pub fn new_template(out: &mut Local, isolate: *mut c_void, callback: *mut c_void, kernel: *mut c_void) -> bool;

    /// Mutates the `out` argument provided to refer to a newly created `v8::FunctionTemplate`.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "Neon_Fun_Template_New2"]
    pub fn new_template_vm2(out: &mut Local, isolate: *mut c_void, callback: CCallback) -> bool;

    /// Gets the reference to the `v8::Local<v8::External>` handle provided.
    #[link_name = "Neon_Fun_GetDynamicCallback"]
    pub fn get_dynamic_callback(obj: Local) -> *mut c_void;

    /// Calls the function provided (`fun`) and mutates the `out` argument provided to refer to the
    /// result of the function call. Returns `false` if the result of the call was empty.
    #[link_name = "Neon_Fun_Call"]
    pub fn call(out: &mut Local, isolate: *mut c_void, fun: Local, this: Local, argc: i32, argv: *mut c_void) -> bool;

    /// Makes a constructor call to with the function provided (`fun`) and mutates the `out`
    /// argument provided to refer to the result of the constructor call. Returns `false` if the
    /// result of the call was empty.
    #[link_name = "Neon_Fun_Construct"]
    pub fn construct(out: &mut Local, isolate: *mut c_void, fun: Local, argc: i32, argv: *mut c_void) -> bool;

}
