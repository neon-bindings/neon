//! Fundamental definitions for mapping to the V8 memory space.

pub use neon_sys::{
    EscapableHandleScope, FunctionCallbackInfo, HandleScope, InheritedHandleScope, Isolate, Local,
};
