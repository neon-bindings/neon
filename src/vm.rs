//! Abstractions representing the JavaScript virtual machine and its control flow.

pub use internal::vm::{Call, FunctionCall, Arguments, Module, Throw, VmResult, JsResult, Lock};
