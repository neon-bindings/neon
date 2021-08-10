//! Types and traits representing binary JavaScript data.

#[cfg(feature = "legacy-runtime")]
pub(crate) mod legacy;

#[cfg(feature = "legacy-runtime")]
pub use legacy::*;

#[cfg(feature = "napi-1")]
pub(crate) mod napi;

#[cfg(feature = "napi-1")]
pub use napi::*;
