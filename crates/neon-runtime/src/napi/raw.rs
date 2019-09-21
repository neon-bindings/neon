use std::os::raw::c_void;
use std::ptr;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Local {
    pub handle: *mut c_void
}

pub type FunctionCallbackInfo = c_void;

pub type Isolate = c_void;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HandleScope {
    pub word: *mut c_void
}

impl HandleScope {
    pub fn new() -> Self { Self { word: ptr::null_mut() } }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EscapableHandleScope {
    pub word: *mut c_void
}

impl EscapableHandleScope {
    pub fn new() -> Self { Self { word: ptr::null_mut() } }
}

#[derive(Clone, Copy)]
pub struct InheritedHandleScope;
