//! Facilities for creating and throwing `Error`s.

use raw::Local;

extern "system" {

    /// Throws an `Error` object in the current context.
    #[link_name = "NeonSys_Error_Throw"]
    pub fn throw(val: Local);

    /// Mutates the `out` argument provided to refer to a newly created `Error` object.
    #[link_name = "NeonSys_Error_NewError"]
    pub fn new_error(out: &mut Local, msg: Local);

    /// Mutates the `out` argument provided to refer to a newly created `TypeError` object.
    #[link_name = "NeonSys_Error_NewTypeError"]
    pub fn new_type_error(out: &mut Local, msg: Local);

    /// Mutates the `out` argument provided to refer to a newly created `ReferenceError` object.
    #[link_name = "NeonSys_Error_NewReferenceError"]
    pub fn new_reference_error(out: &mut Local, msg: Local);

    /// Mutates the `out` argument provided to refer to a newly created `RangeError` object.
    #[link_name = "NeonSys_Error_NewRangeError"]
    pub fn new_range_error(out: &mut Local, msg: Local);

    /// Mutates the `out` argument provided to refer to a newly created `SyntaxError` object.
    #[link_name = "NeonSys_Error_NewSyntaxError"]
    pub fn new_syntax_error(out: &mut Local, msg: Local);

    /// Throws an `Error` object in the current context.
    #[link_name = "NeonSys_Error_ThrowErrorFromCString"]
    pub fn throw_error_from_cstring(msg: *const u8);

    /// Throws a `TypeError` object in the current context.
    #[link_name = "NeonSys_Error_ThrowTypeErrorFromCString"]
    pub fn throw_type_error_from_cstring(msg: *const u8);

    /// Throws a `ReferenceError` object in the current context.
    #[link_name = "NeonSys_Error_ThrowReferenceErrorFromCString"]
    pub fn throw_reference_error_from_cstring(msg: *const u8);

    /// Throws a `RangeError` object in the current context.
    #[link_name = "NeonSys_Error_ThrowRangeErrorFromCString"]
    pub fn throw_range_error_from_cstring(msg: *const u8);

    /// Throws a `SyntaxError` object in the current context.
    #[link_name = "NeonSys_Error_ThrowSyntaxErrorFromCString"]
    pub fn throw_syntax_error_from_cstring(msg: *const u8);

}
