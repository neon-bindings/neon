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

pub unsafe extern "C" fn escape(_out: &mut Local, _scope: *mut EscapableHandleScope, _value: Local) { unimplemented!() }

pub unsafe extern "C" fn chained(_out: *mut c_void, _closure: *mut c_void, _callback: extern fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void), _parent_scope: *mut c_void) { unimplemented!() }

pub unsafe extern "C" fn nested(_out: *mut c_void, _closure: *mut c_void, _callback: extern fn(&mut c_void, *mut c_void, *mut c_void), _realm: *mut c_void) { unimplemented!() }

pub unsafe extern "C" fn enter(_scope: &mut HandleScope, _isolate: *mut c_void) { unimplemented!() }

pub unsafe extern "C" fn exit(_scope: &mut HandleScope) { unimplemented!() }

pub unsafe extern "C" fn enter_escapable(_scope: &mut EscapableHandleScope, _isolate: *mut c_void) { unimplemented!() }

pub unsafe extern "C" fn exit_escapable(_scope: &mut EscapableHandleScope) { unimplemented!() }

pub unsafe extern "C" fn size() -> usize { unimplemented!() }

pub unsafe extern "C" fn alignment() -> usize { unimplemented!() }

pub unsafe extern "C" fn escapable_size() -> usize { unimplemented!() }

pub unsafe extern "C" fn escapable_alignment() -> usize { unimplemented!() }

pub unsafe extern "C" fn get_global(_isolate: *mut c_void, _out: &mut Local) { unimplemented!() }
