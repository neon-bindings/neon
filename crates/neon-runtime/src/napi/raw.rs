//! Fundamental definitions for mapping to the V8 memory space.

use std::os::raw::c_void;
use std::mem;
use std::ptr;

/// A V8 `Local` handle.
///
/// `Local` handles get associated to a V8 `HandleScope` container. Note: Node.js creates a
/// `HandleScope` right before calling functions in native addons.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Local {
    pub handle: *mut c_void
}

/// Represents the details of how the function was called from JavaScript.
///
/// It contains the arguments used to invoke the function, the isolate reference, the `this` object
/// the function is bound to and a mechanism to return a value to the caller.
pub type FunctionCallbackInfo = c_void;

/// Represents an instance of the V8 runtime.
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
