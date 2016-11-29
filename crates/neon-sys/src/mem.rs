//! A helper function for comparing `Local` handles.
use raw::Local;

extern "system" {

    /// Indicates if two `Local` handles are the same.
    #[link_name = "NeonSys_Mem_SameHandle"]
    pub fn same_handle(h1: Local, h2: Local) -> bool;

}
