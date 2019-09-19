//! Facilities for working with `v8::ArrayBuffer`s.

/// Mutates the `out` argument provided to refer to a newly created `v8::ArrayBuffer` object.
/// Returns `false` if the value couldn't be created.
pub use neon_sys::Neon_ArrayBuffer_New as new;

/// Mutates the `base_out` and `size_out` arguments to access the data of a `v8::ArrayBuffer` object.
pub use neon_sys::Neon_ArrayBuffer_Data as data;
