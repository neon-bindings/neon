//! Facilities for creating and throwing JS errors.

use raw::{Isolate, Persistent};

extern "C" {

    /// Throws an `Error` object in the current context.
    #[link_name = "Neon_Error_Throw"]
    pub fn throw(val: &Persistent);

    /// Initializes the `out` argument provided to refer to a newly created `Error` object.
    #[link_name = "Neon_Error_NewError"]
    pub fn new_error(out: &Persistent, isolate: *mut Isolate, msg: &Persistent);

    /// Initializes the `out` argument provided to refer to a newly created `Error` object.
    #[link_name = "Neon_Error_NewTypeError"]
    pub fn new_type_error(out: &Persistent, isolate: *mut Isolate, msg: &Persistent);

    /// Initializes the `out` argument provided to refer to a newly created `Error` object.
    #[link_name = "Neon_Error_NewRangeError"]
    pub fn new_range_error(out: &Persistent, isolate: *mut Isolate, msg: &Persistent);

    /// Throws an `Error` object in the current context.
    #[link_name = "Neon_Error_ThrowErrorFromUtf8"]
    pub fn throw_error_from_utf8(msg: *const u8, len: i32);

}
