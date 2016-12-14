//! Facilities for working with `HandleScope`s and `EscapableHandleScope`s.

use std::os::raw::c_void;
use raw::{HandleScope, EscapableHandleScope, Local};

extern "system" {

    /// Mutates the `out` argument provided to refer to the newly escaped `Local` value.
    #[link_name = "NeonSys_Scope_Escape"]
    pub fn escape(out: &mut Local, scope: *mut EscapableHandleScope, value: Local);

    /// Creates an `EscapableHandleScope` and calls the `callback` provided with the argument
    /// signature `(out, parent_scope, &v8_scope, closure)`.
    #[link_name = "NeonSys_Scope_Chained"]
    pub fn chained(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void), parent_scope: *mut c_void);

    /// Creates a `HandleScope` and calls the `callback` provided with the argument signature
    /// `(out, realm, closure)`.
    #[link_name = "NeonSys_Scope_Nested"]
    pub fn nested(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void), realm: *mut c_void);

    /// Instantiates a new `HandleScope`.
    #[link_name = "NeonSys_Scope_Enter"]
    pub fn enter(scope: &mut HandleScope, isolate: *mut c_void);

    /// Destructs a `HandleScope`.
    #[link_name = "NeonSys_Scope_Exit"]
    pub fn exit(scope: &mut HandleScope);

    /// Gets the size of a `HandleScope`.
    #[link_name = "NeonSys_Scope_Sizeof"]
    pub fn size() -> usize;

    /// Gets the alignment requirement of a `HandleScope`.
    #[link_name = "NeonSys_Scope_Alignof"]
    pub fn alignment() -> usize;

    /// Gets the size of a `EscapableHandleScope`.
    #[link_name = "NeonSys_Scope_SizeofEscapable"]
    pub fn escapable_size() -> usize;

    /// Gets the alignment requirement of a `EscapableHandleScope`.
    #[link_name = "NeonSys_Scope_AlignofEscapable"]
    pub fn escapable_alignment() -> usize;

}
