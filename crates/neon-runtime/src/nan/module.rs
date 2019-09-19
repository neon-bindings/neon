//! A helper function for initializing a module.

/// Creates a new `v8::HandleScope` and calls `callback` provided with the argument signature
/// `(kernel, exports, scope, vm)`.
pub use neon_sys::Neon_Module_ExecKernel as exec_kernel;

pub use neon_sys::Neon_Module_ExecCallback as exec_callback;

pub use neon_sys::Neon_Module_GetVersion as get_version;
