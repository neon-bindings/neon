//! Facilities for working with `v8::HandleScope`s and `v8::EscapableHandleScope`s.

use std::os::raw::c_void;
use raw::{HandleScope, EscapableHandleScope, InheritedHandleScope, Local, Isolate};

pub trait Root {
    unsafe fn allocate() -> Self;
    unsafe fn enter(&mut self, *mut Isolate);
    unsafe fn exit(&mut self);
}

impl Root for HandleScope {
    unsafe fn allocate() -> Self { HandleScope::new() }
    unsafe fn enter(&mut self, isolate: *mut Isolate) {
        enter(self, isolate)
    }
    unsafe fn exit(&mut self) {
        exit(self)
    }
}

impl Root for EscapableHandleScope {
    unsafe fn allocate() -> Self { EscapableHandleScope::new() }
    unsafe fn enter(&mut self, isolate: *mut Isolate) {
        enter_escapable(self, isolate)
    }
    unsafe fn exit(&mut self) {
        exit_escapable(self)
    }
}

impl Root for InheritedHandleScope {
    unsafe fn allocate() -> Self { InheritedHandleScope }
    unsafe fn enter(&mut self, _: *mut Isolate) { }
    unsafe fn exit(&mut self) { }
}

// FIXME(napi): #[link_name = "Neon_Scope_Escape"]
pub extern "C" fn escape(out: &mut Local, scope: *mut EscapableHandleScope, value: Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_Chained"]
pub extern "C" fn chained(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void), parent_scope: *mut c_void) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_Nested"]
pub extern "C" fn nested(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void), realm: *mut c_void) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_Enter"]
pub extern "C" fn enter(scope: &mut HandleScope, isolate: *mut c_void) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_Exit"]
pub extern "C" fn exit(scope: &mut HandleScope) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_Enter_Escapable"]
pub extern "C" fn enter_escapable(scope: &mut EscapableHandleScope, isolate: *mut c_void) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_Exit_Escapable"]
pub extern "C" fn exit_escapable(scope: &mut EscapableHandleScope) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_Sizeof"]
pub extern "C" fn size() -> usize { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_Alignof"]
pub extern "C" fn alignment() -> usize { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_SizeofEscapable"]
pub extern "C" fn escapable_size() -> usize { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_AlignofEscapable"]
pub extern "C" fn escapable_alignment() -> usize { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Scope_GetGlobal"]
pub extern "C" fn get_global(isolate: *mut c_void, out: &mut Local) { unimplemented!() }
