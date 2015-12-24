use std::os::raw::c_void;
use raw::{EscapableHandleScope, Local};

extern "system" {

    #[link_name = "NeonSys_Scope_Escape"]
    pub fn escape(out: &mut Local, scope: *mut EscapableHandleScope, value: Local);

    #[link_name = "NeonSys_Scope_Chained"]
    pub fn chained(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void), parent_scope: *mut c_void);

    #[link_name = "NeonSys_Scope_Nested"]
    pub fn nested(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void), realm: *mut c_void);

}
