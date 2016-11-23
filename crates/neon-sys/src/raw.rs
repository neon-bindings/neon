use std::os::raw::c_void;
use std::mem;

/// Wrapper around a v8 Local.
///
/// An object reference managed by the v8 garbage collector. All objects are stored in handles 
/// which are known by the garbage collector and updated whenever an object moves.
/// Local handles are light-weight and transient and typically used in local operations.
///
/// Edited from http://v8.paulfryzel.com/docs/master/classv8_1_1_local.html
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Local {
    pub handle: *mut c_void
}

pub type FunctionCallbackInfo = c_void;

pub type Isolate = c_void;

const HANDLE_SCOPE_SIZE: usize = 24;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HandleScope {
    pub align_to_pointer: [*mut c_void; 0],
    pub fields: [u8; HANDLE_SCOPE_SIZE]
}

impl HandleScope {
    pub fn new() -> HandleScope { unsafe { mem::zeroed() } }
}

const ESCAPABLE_HANDLE_SCOPE_SIZE: usize = 32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EscapableHandleScope {
    pub align_to_pointer: [*mut c_void; 0],
    pub fields: [u8; ESCAPABLE_HANDLE_SCOPE_SIZE]
}

impl EscapableHandleScope {
    pub fn new() -> EscapableHandleScope { unsafe { mem::zeroed() } }
}
