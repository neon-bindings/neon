//! Abstractions for temporarily _rooting_ handles to managed JavaScript memory.

pub use internal::scope::{Scope, RootScope, NestedScope, ChainedScope};
