//! Facilities for running a callback in the libuv main thread.

/// Creates a new thread safe callback which can be used to execute a callback in the libuv main thread
pub use neon_sys::Neon_ThreadSafeCallback_New as new;
/// Executes the given callback in the libuv main thread
pub use neon_sys::Neon_ThreadSafeCallback_Call as call;
// Free the thread safe callback and any memory hold
pub use neon_sys::Neon_ThreadSafeCallback_Delete as delete;
