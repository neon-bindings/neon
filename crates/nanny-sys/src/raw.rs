use std::os::raw::c_void;

#[repr(C)]
#[allow(raw_pointer_derive)]
#[derive(Clone)]
pub struct Local {
    pub handle: *mut c_void
}

pub type FunctionCallbackInfo = c_void;
