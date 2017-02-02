//! Fundamental definitions for mapping to the V8 memory space.

use std::os::raw::c_void;
use std::mem;

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

const HANDLE_SCOPE_SIZE: usize = 24;

/// A V8 `HandleScope`.
///
/// A `HandleScope` contains `Local` handles. `HandleScope`s are used by V8 to help the garbage
/// collector do its bookkeeping. Once a new `HandleScope` is created all subsequently created
/// `Local` handles will be contained in it.
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

/// A V8 `EscapableHandleScope`.
///
/// A `EscapableHandleScope` is like `HandleScope` but also allows us to push `Local` handles out
/// to the previous `HandleScope`, permitting the `Local` value to remain rooted longer than the
/// `EscapableHandleScope` where it was intially rooted.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EscapableHandleScope {
    pub align_to_pointer: [*mut c_void; 0],
    pub fields: [u8; ESCAPABLE_HANDLE_SCOPE_SIZE]
}

impl EscapableHandleScope {
    pub fn new() -> EscapableHandleScope { unsafe { mem::zeroed() } }
}
