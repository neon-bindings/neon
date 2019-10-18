//! Facilities for running a callback in the libuv main thread.

/// Creates a new event handler which can be used to execute a callback in the libuv main thread
pub use neon_sys::Neon_EventHandler_New as new;
pub use neon_sys::Neon_EventHandler_Bind as bind;
/// Executes the given callback in the libuv main thread
pub use neon_sys::Neon_EventHandler_Schedule as schedule;
// Free the thread safe callback and any memory hold
pub use neon_sys::Neon_EventHandler_Delete as delete;
