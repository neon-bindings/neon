//! A convenience module that re-exports the most commonly-used Neon APIs.

pub use crate::borrow::{Borrow, BorrowMut};
pub use crate::context::{
    CallContext, CallKind, ComputeContext, Context, ExecuteContext, FunctionContext, MethodContext,
    ModuleContext, TaskContext,
};
pub use crate::handle::Handle;
pub use crate::object::{Class, Object};
pub use crate::result::{JsResult, JsResultExt, NeonResult};
pub use crate::task::Task;
pub use crate::types::{
    BinaryData, JsArray, JsArrayBuffer, JsBoolean, JsBuffer, JsError, JsFunction, JsNull, JsNumber,
    JsObject, JsString, JsUndefined, JsValue, Value,
};
