use std::os::raw::c_void;
use raw::Local;

extern "system" {

    #[link_name = "NeonSys_Module_ExecBody"]
    pub fn ExecBody(kernel: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void), exports: Local, scope: *mut c_void);

}
