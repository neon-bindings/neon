//! Facilities for creating and throwing JS errors.

use crate::raw::{Isolate, Local};

/// Throws an `Error` object in the current context.
pub unsafe fn throw(_: Isolate, val: Local) {
    neon_sys::Neon_Error_Throw(val)
}

/// Mutates the `out` argument provided to refer to a newly created `Error` object.
pub unsafe fn new_error(_: Isolate, out: &mut Local, msg: Local) {
    neon_sys::Neon_Error_NewError(out, msg)
}

/// Mutates the `out` argument provided to refer to a newly created `TypeError` object.
pub unsafe fn new_type_error(_: Isolate, out: &mut Local, msg: Local) {
    neon_sys::Neon_Error_NewTypeError(out, msg)
}

/// Mutates the `out` argument provided to refer to a newly created `RangeError` object.
pub unsafe fn new_range_error(_: Isolate, out: &mut Local, msg: Local) {
    neon_sys::Neon_Error_NewRangeError(out, msg)
}

/// Throws an `Error` object in the current context.
pub unsafe fn throw_error_from_utf8(_: Isolate, msg: *const u8, len: i32) {
    neon_sys::Neon_Error_ThrowErrorFromUtf8(msg, len)
}
