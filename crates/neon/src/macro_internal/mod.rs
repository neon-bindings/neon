//! Internals needed by macros. These have to be exported for the macros to work

#[cfg(feature = "export")]
pub use self::export::*;

#[cfg(all(feature = "napi-6", feature = "futures"))]
pub use self::futures::*;

#[cfg(feature = "export")]
mod export;

#[cfg(all(feature = "napi-6", feature = "futures"))]
mod futures;

#[cfg(not(feature = "export"))]
pub use crate::context::internal::initialize_module;
