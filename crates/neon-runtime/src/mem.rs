//! A helper function for comparing `v8::Local` handles.

use std::os::raw::c_void;
use raw::Local;

extern "C" {

    /// Indicates if two `v8::Local` handles are the same.
    #[link_name = "Neon_Mem_SameHandle"]
    pub fn same_handle(h1: Local, h2: Local) -> bool;

    #[link_name = "Neon_Mem_NewPersistent"]
    pub fn new_persistent(h: Local) -> *mut c_void;

    #[link_name = "Neon_Mem_New"]
    pub fn new(out: &mut Local, p: *mut c_void);

}
