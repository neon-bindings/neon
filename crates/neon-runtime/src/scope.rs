//! Facilities for working with `v8::HandleScope`s and `v8::EscapableHandleScope`s.

use raw::{EscapableHandleScope, HandleScope, InheritedHandleScope, Isolate, Local};
use std::os::raw::c_void;

pub trait Root {
    unsafe fn allocate() -> Self;
    unsafe fn enter(&mut self, *mut Isolate);
    unsafe fn exit(&mut self);
}

impl Root for HandleScope {
    unsafe fn allocate() -> Self {
        HandleScope::new()
    }
    unsafe fn enter(&mut self, isolate: *mut Isolate) {
        enter(self, isolate)
    }
    unsafe fn exit(&mut self) {
        exit(self)
    }
}

impl Root for EscapableHandleScope {
    unsafe fn allocate() -> Self {
        EscapableHandleScope::new()
    }
    unsafe fn enter(&mut self, isolate: *mut Isolate) {
        enter_escapable(self, isolate)
    }
    unsafe fn exit(&mut self) {
        exit_escapable(self)
    }
}

impl Root for InheritedHandleScope {
    unsafe fn allocate() -> Self {
        InheritedHandleScope
    }
    unsafe fn enter(&mut self, _: *mut Isolate) {}
    unsafe fn exit(&mut self) {}
}

extern "C" {

    /// Mutates the `out` argument provided to refer to the newly escaped `v8::Local` value.
    #[link_name = "Neon_Scope_Escape"]
    pub fn escape(out: &mut Local, scope: *mut EscapableHandleScope, value: Local);

    /// Creates a `v8::EscapableHandleScope` and calls the `callback` provided with the argument
    /// signature `(out, parent_scope, &v8_scope, closure)`.
    #[link_name = "Neon_Scope_Chained"]
    pub fn chained(
        out: *mut c_void,
        closure: *mut c_void,
        callback: extern "C" fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void),
        parent_scope: *mut c_void,
    );

    /// Creates a `v8::HandleScope` and calls the `callback` provided with the argument signature
    /// `(out, realm, closure)`.
    #[link_name = "Neon_Scope_Nested"]
    pub fn nested(
        out: *mut c_void,
        closure: *mut c_void,
        callback: extern "C" fn(&mut c_void, *mut c_void, *mut c_void),
        realm: *mut c_void,
    );

    /// Instantiates a new `v8::HandleScope`.
    #[link_name = "Neon_Scope_Enter"]
    pub fn enter(scope: &mut HandleScope, isolate: *mut c_void);

    /// Destructs a `v8::HandleScope`.
    #[link_name = "Neon_Scope_Exit"]
    pub fn exit(scope: &mut HandleScope);

    /// Instantiates a new `v8::HandleScope`.
    #[link_name = "Neon_Scope_Enter_Escapable"]
    pub fn enter_escapable(scope: &mut EscapableHandleScope, isolate: *mut c_void);

    /// Destructs a `v8::HandleScope`.
    #[link_name = "Neon_Scope_Exit_Escapable"]
    pub fn exit_escapable(scope: &mut EscapableHandleScope);

    /// Gets the size of a `v8::HandleScope`.
    #[link_name = "Neon_Scope_Sizeof"]
    pub fn size() -> usize;

    /// Gets the alignment requirement of a `v8::HandleScope`.
    #[link_name = "Neon_Scope_Alignof"]
    pub fn alignment() -> usize;

    /// Gets the size of a `v8::EscapableHandleScope`.
    #[link_name = "Neon_Scope_SizeofEscapable"]
    pub fn escapable_size() -> usize;

    /// Gets the alignment requirement of a `v8::EscapableHandleScope`.
    #[link_name = "Neon_Scope_AlignofEscapable"]
    pub fn escapable_alignment() -> usize;

    /// Mutates the `out` argument provided to refer to the `v8::Local` value of the `global`
    /// object
    #[link_name = "Neon_Scope_GetGlobal"]
    pub fn get_global(isolate: *mut c_void, out: &mut Local);

}
