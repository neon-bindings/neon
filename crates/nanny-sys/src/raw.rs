use std::os::raw::c_void;

#[repr(C)]
#[allow(raw_pointer_derive)]
#[derive(Clone, Copy)]
pub struct Local {
    pub handle: *mut c_void
}

pub type FunctionCallbackInfo = c_void;

pub type Isolate = c_void;

pub type EscapableHandleScope = c_void;
