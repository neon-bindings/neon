//! Internals needed by macros. These have to be exported for the macros to work
pub use crate::context::internal::{initialize_module, Env};
#[cfg(feature = "legacy-runtime")]
pub use crate::handle::internal::TransparentNoCopyWrapper;
/// but are subject to change and should never be explicitly used.

#[cfg(feature = "legacy-runtime")]
// Used by the class macro.
pub use crate::object::class::internal::{
    AllocateCallback, ConstructCallback, ConstructorCallCallback, MethodCallback,
};

// An alias for neon_runtime so macros can refer to it.
pub mod runtime {
    pub use neon_runtime::*;
}
