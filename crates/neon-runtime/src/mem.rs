//! A helper function for comparing `v8::Local` handles.
use raw::{Local, Persistent};

extern "C" {

    /// Indicates if two `v8::Local` handles are the same.
    #[link_name = "Neon_Mem_SameHandle"]
    pub fn same_handle(h1: Local, h2: Local) -> bool;

    /// Initializes a pointer as a `v8::Persistent`.
    #[link_name = "Neon_Mem_NewPersistent"]
    pub fn new_persistent(out: &mut Persistent);

    /// Deallocates a `v8::Persistent`.
    #[link_name = "Neon_Mem_DropPersistent"]
    pub fn drop_persistent(p: &mut Persistent);

    /// Resets a `v8::Persistent` to the given handle.
    #[link_name = "Neon_Mem_ResetPersistent"]
    pub fn reset_persistent(p: &Persistent, h: Local);

}
