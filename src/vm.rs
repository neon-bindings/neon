//! Abstractions representing the JavaScript virtual machine and its control flow.

pub use internal::vm::{Call, Arguments, Module, Throw, VmResult, JsResult, Lock, lock};
