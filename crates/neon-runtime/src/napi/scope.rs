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

pub extern "C" fn escape(out: &mut Local, scope: *mut EscapableHandleScope, value: Local) { unimplemented!() }

pub extern "C" fn chained(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void), parent_scope: *mut c_void) { unimplemented!() }

pub extern "C" fn nested(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void), realm: *mut c_void) { unimplemented!() }

pub extern "C" fn enter(scope: &mut HandleScope, isolate: *mut c_void) { unimplemented!() }

pub extern "C" fn exit(scope: &mut HandleScope) { unimplemented!() }

pub extern "C" fn enter_escapable(scope: &mut EscapableHandleScope, isolate: *mut c_void) { unimplemented!() }

pub extern "C" fn exit_escapable(scope: &mut EscapableHandleScope) { unimplemented!() }

pub extern "C" fn size() -> usize { unimplemented!() }

pub extern "C" fn alignment() -> usize { unimplemented!() }

pub extern "C" fn escapable_size() -> usize { unimplemented!() }

pub extern "C" fn escapable_alignment() -> usize { unimplemented!() }

pub extern "C" fn get_global(isolate: *mut c_void, out: &mut Local) { unimplemented!() }
