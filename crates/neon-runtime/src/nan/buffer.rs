//! Facilities for working with `node::Buffer`s.

/// Mutates the `out` argument provided to refer to a newly created and zero-filled `node::Buffer` object.
/// Returns `false` if the value couldn't be created.
pub use neon_sys::Neon_Buffer_New as new;

/// Mutates the `out` argument provided to refer to a newly created `node::Buffer` object.
/// Returns `false` if the value couldn't be created.
pub use neon_sys::Neon_Buffer_Uninitialized as uninitialized;

/// Mutates the `base_out` and `size_out` arguments to access the data of a `node::Buffer` object.
pub use neon_sys::Neon_Buffer_Data as data;
