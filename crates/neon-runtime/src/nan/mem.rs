//! A helper function for comparing `v8::Local` handles.

/// Indicates if two `v8::Local` handles are the same.
pub use neon_sys::Neon_Mem_SameHandle as same_handle;
