use std::os::raw::c_void;
use std::mem;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Local {
    pub handle: *mut c_void
}

pub type FunctionCallbackInfo = c_void;

pub type Isolate = c_void;

pub const HANDLE_SCOPE_SIZE: usize = 24;
pub const ESCAPABLE_HANDLE_SCOPE_SIZE: usize = 32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HandleScope {
    pub align_to_pointer: [*mut c_void; 0],
    pub fields: [u8; HANDLE_SCOPE_SIZE]
}

impl HandleScope {
    pub fn new() -> HandleScope { unsafe { mem::zeroed() } }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EscapableHandleScope {
    pub align_to_pointer: [*mut c_void; 0],
    pub fields: [u8; ESCAPABLE_HANDLE_SCOPE_SIZE]
}

impl EscapableHandleScope {
    pub fn new() -> EscapableHandleScope { unsafe { mem::zeroed() } }
}
