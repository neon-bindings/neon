//! Facilities for working with `v8::Array`s.

/// Mutates the `out` argument provided to refer to a newly created `v8::Array`.
pub use neon_sys::Neon_Array_New as new;

/// Gets the length of an `v8::Array`.
pub use neon_sys::Neon_Array_Length as len;
