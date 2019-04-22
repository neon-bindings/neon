//! A helper function for initializing a module.

use call::CCallback;
use raw::Local;
use std::os::raw::c_void;

extern "C" {

    /// Creates a new `v8::HandleScope` and calls `callback` provided with the argument signature
    /// `(kernel, exports, scope, vm)`.
    #[link_name = "Neon_Module_ExecKernel"]
    pub fn exec_kernel(
        kernel: *mut c_void,
        callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void),
        exports: Local,
        scope: *mut c_void,
        vm: *mut c_void,
    );

    #[link_name = "Neon_Module_ExecCallback"]
    pub fn exec_callback(callback: CCallback, exports: Local, vm: *mut c_void);

    #[link_name = "Neon_Module_GetVersion"]
    pub fn get_version() -> i32;

}
