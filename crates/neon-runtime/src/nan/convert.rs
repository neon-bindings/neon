//! Helper functions for converting `v8::Local` values.

/// Casts the value provided to a `v8::Object` and mutates the `out` argument provided to refer
/// to `v8::Local` handle of the converted value. Returns `false` if the conversion didn't
/// succeed.
pub use neon_sys::Neon_Convert_ToObject as to_object;

/// Casts the value provided to a `v8::String` and mutates the `out` argument provided to refer
/// to `v8::Local` handle of the converted value. Returns `false` if the conversion didn't
/// succeed.
pub use neon_sys::Neon_Convert_ToString as to_string;
