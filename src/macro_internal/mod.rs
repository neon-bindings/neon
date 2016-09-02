//! Internals needed by macros. These have to be exported for the macros to work
/// but are subject to change and should never be explicitly used.

// Used by the class macro.
pub use internal::js::class::{AllocateKernel, ConstructKernel, ConstructorCallKernel, MethodKernel};

// An alias for neon_sys so macros can refer to it.
pub mod sys {
    pub use neon_sys::*;
}
