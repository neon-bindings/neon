//! Internals needed by macros. These have to be exported for the macros to work
pub use context::internal::{initialize_module, Env};
/// but are subject to change and should never be explicitly used.

#[cfg(feature = "legacy-runtime")]
// Used by the class macro.
pub use object::class::internal::{
    AllocateCallback, ConstructCallback, ConstructorCallCallback, MethodCallback,
};

// An alias for neon_runtime so macros can refer to it.
pub mod runtime {
    pub use neon_runtime::*;
}
