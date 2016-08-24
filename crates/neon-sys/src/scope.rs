use std::os::raw::c_void;
use raw::{HandleScope, EscapableHandleScope, Local};

extern "system" {

    #[link_name = "NeonSys_Scope_Escape"]
    pub fn escape(out: &mut Local, scope: *mut EscapableHandleScope, value: Local);

    #[link_name = "NeonSys_Scope_Chained"]
    pub fn chained(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void), parent_scope: *mut c_void);

    #[link_name = "NeonSys_Scope_Nested"]
    pub fn nested(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void), realm: *mut c_void);

    #[link_name = "NeonSys_Scope_Enter"]
    pub fn enter(scope: &mut HandleScope, isolate: *mut c_void);

    #[link_name = "NeonSys_Scope_Exit"]
    pub fn exit(scope: &mut HandleScope);

    #[link_name = "NeonSys_Scope_Sizeof"]
    pub fn size() -> usize;

    #[link_name = "NeonSys_Scope_SizeofEscapable"]
    pub fn escapable_size() -> usize;

}
