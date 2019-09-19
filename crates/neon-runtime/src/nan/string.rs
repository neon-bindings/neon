//! Facilities for working with `v8::String`s.

/// Mutates the `out` argument provided to refer to a newly created `v8::String`. Returns
/// `false` if the value couldn't be created.
pub use neon_sys::Neon_String_New as new;

/// Gets the length of a `v8::String`.
pub use neon_sys::Neon_String_Utf8Length as utf8_len;

/// Writes data to a `v8::String` and returns the number of bytes writen.
pub use neon_sys::Neon_String_Data as data;
