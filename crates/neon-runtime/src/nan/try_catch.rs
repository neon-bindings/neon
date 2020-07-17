/// Wraps a computation with an RAII-allocated Nan::TryCatch.
pub use neon_sys::Neon_TryCatch_With as with;

pub use neon_sys::TryCatchControl;
