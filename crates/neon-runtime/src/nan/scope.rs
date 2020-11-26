//! Facilities for working with `v8::HandleScope`s and `v8::EscapableHandleScope`s.

use crate::raw::{HandleScope, EscapableHandleScope, InheritedHandleScope, Isolate};

pub trait Root {
    unsafe fn allocate() -> Self;
    unsafe fn enter(&mut self, isolate: Isolate);
    unsafe fn exit(&mut self, isolate: Isolate);
}

impl Root for HandleScope {
    unsafe fn allocate() -> Self { HandleScope::new() }
    unsafe fn enter(&mut self, isolate: Isolate) {
        enter(self, isolate)
    }
    unsafe fn exit(&mut self, _: Isolate) {
        exit(self)
    }
}

impl Root for EscapableHandleScope {
    unsafe fn allocate() -> Self { EscapableHandleScope::new() }
    unsafe fn enter(&mut self, isolate: Isolate) {
        enter_escapable(self, isolate)
    }
    unsafe fn exit(&mut self, _: Isolate) {
        exit_escapable(self)
    }
}

impl Root for InheritedHandleScope {
    unsafe fn allocate() -> Self { InheritedHandleScope }
    unsafe fn enter(&mut self, _: Isolate) { }
    unsafe fn exit(&mut self, _: Isolate) { }
}

/// Mutates the `out` argument provided to refer to the newly escaped `v8::Local` value.
pub use neon_sys::Neon_Scope_Escape as escape;

/// Creates a `v8::EscapableHandleScope` and calls the `callback` provided with the argument
/// signature `(out, parent_scope, &v8_scope, closure)`.
pub use neon_sys::Neon_Scope_Chained as chained;

/// Creates a `v8::HandleScope` and calls the `callback` provided with the argument signature
/// `(out, realm, closure)`.
pub use neon_sys::Neon_Scope_Nested as nested;

/// Instantiates a new `v8::HandleScope`.
pub use neon_sys::Neon_Scope_Enter as enter;

/// Destructs a `v8::HandleScope`.
pub use neon_sys::Neon_Scope_Exit as exit;

/// Instantiates a new `v8::HandleScope`.
pub use neon_sys::Neon_Scope_Enter_Escapable as enter_escapable;

/// Destructs a `v8::HandleScope`.
pub use neon_sys::Neon_Scope_Exit_Escapable as exit_escapable;

/// Gets the size of a `v8::HandleScope`.
pub use neon_sys::Neon_Scope_Sizeof as size;

/// Gets the alignment requirement of a `v8::HandleScope`.
pub use neon_sys::Neon_Scope_Alignof as alignment;

/// Gets the size of a `v8::EscapableHandleScope`.
pub use neon_sys::Neon_Scope_SizeofEscapable as escapable_size;

/// Gets the alignment requirement of a `v8::EscapableHandleScope`.
pub use neon_sys::Neon_Scope_AlignofEscapable as escapable_alignment;

/// Mutates the `out` argument provided to refer to the `v8::Local` value of the `global`
/// object
pub use neon_sys::Neon_Scope_GetGlobal as get_global;
