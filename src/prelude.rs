//! A convenience module that re-exports the most commonly-used Neon APIs.

pub use borrow::{Borrow, BorrowMut};
pub use context::{
    CallContext, CallKind, ComputeContext, Context, ExecuteContext, FunctionContext, MethodContext,
    ModuleContext, TaskContext,
};
pub use handle::Handle;
pub use object::{Class, Object};
pub use result::{JsResult, JsResultExt, NeonResult};
pub use task::Task;
pub use types::{
    BinaryData, JsArray, JsArrayBuffer, JsBoolean, JsBuffer, JsError, JsFunction, JsNull, JsNumber,
    JsObject, JsString, JsUndefined, JsValue, Value,
};
