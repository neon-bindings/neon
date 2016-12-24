//! A helper function for comparing `v8::Local` handles.
use raw::Local;

extern "C" {

    /// Indicates if two `v8::Local` handles are the same.
    #[link_name = "NeonSys_Mem_SameHandle"]
    pub fn same_handle(h1: Local, h2: Local) -> bool;

}
