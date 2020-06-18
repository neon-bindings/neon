/// Creates a `Nan::TryCatch`
pub use neon_sys::Neon_TryCatch_New as new;

/// Returns a boolean indicating if an exception has been caught
pub use neon_sys::Neon_TryCatch_HasCaught as has_caught;

/// Returns an exception if one has been caught
pub use neon_sys::Neon_TryCatch_Exception as exception;

/// Deletes an instance of `Nan::TryCatch`
pub use neon_sys::Neon_TryCatch_Delete as delete;
