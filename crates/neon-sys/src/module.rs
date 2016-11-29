//! A helper function for initializing a module.

use std::os::raw::c_void;
use raw::Local;

extern "system" {

    /// Creates a new `v8::HandleScope` and calls `callback` provided with the argument signature
    /// `(kernal, exports, scope)`.
    #[link_name = "NeonSys_Module_ExecKernel"]
    pub fn exec_kernel(kernel: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void), exports: Local, scope: *mut c_void);

}
