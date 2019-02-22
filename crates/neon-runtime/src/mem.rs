//! A helper function for comparing `v8::Local` handles.
use raw::{Local, Persistent};

extern "C" {

    /// Initializes a pointer as a `v8::Persistent`.
    #[link_name = "Neon_Mem_NewPersistent"]
    pub fn new_persistent(out: &mut Persistent);

    /// Deallocates a `v8::Persistent`.
    #[link_name = "Neon_Mem_DropPersistent"]
    pub fn drop_persistent(p: &mut Persistent);

    /// Resets a `v8::Persistent` to the given handle.
    #[link_name = "Neon_Mem_ResetPersistent"]
    pub fn reset_persistent(p: &Persistent, h: Local);

    /// Initializes a local handle with the value of a persistent handle.
    #[link_name = "Neon_Mem_ReadPersistent"]
    pub fn read_persistent(out: &mut Local, p: &Persistent);

}
