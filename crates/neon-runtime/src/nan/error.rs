//! Facilities for creating and throwing JS errors.

/// Throws an `Error` object in the current context.
pub use neon_sys::Neon_Error_Throw as throw;

/// Mutates the `out` argument provided to refer to a newly created `Error` object.
pub use neon_sys::Neon_Error_NewError as new_error;

/// Mutates the `out` argument provided to refer to a newly created `TypeError` object.
pub use neon_sys::Neon_Error_NewTypeError as new_type_error;

/// Mutates the `out` argument provided to refer to a newly created `RangeError` object.
pub use neon_sys::Neon_Error_NewRangeError as new_range_error;

/// Throws an `Error` object in the current context.
pub use neon_sys::Neon_Error_ThrowErrorFromUtf8 as throw_error_from_utf8;
