//! Facilities for working with `v8::HandleScope`s.

use std::os::raw::c_void;
use raw::{Isolate, Persistent};

extern "C" {

    /// Clones one persistent handle into another.
    #[link_name = "Neon_Scope_ClonePersistent"]
    pub fn clone_persistent(isolate: *mut Isolate, to: &Persistent, from: &Persistent);

    /// Creates a `v8::HandleScope` and calls the `callback` provided with the argument signature
    /// `(out, realm, closure)`.
    #[link_name = "Neon_Scope_Nested"]
    pub fn nested(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void), realm: *mut c_void);

    /// Mutates the `out` argument provided to refer to the `v8::Local` value of the `global`
    /// object
    #[link_name = "Neon_Scope_GetGlobal"]
    pub fn get_global(isolate: *mut Isolate, out: &Persistent);

}
