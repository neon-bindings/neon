//! Facilities for running background tasks in the libuv thread pool.

/// Schedules a background task.
pub use neon_sys::Neon_Task_Schedule as schedule;
