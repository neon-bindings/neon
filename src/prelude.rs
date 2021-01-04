//! A convenience module that re-exports the most commonly-used Neon APIs.

pub use handle::Handle;
pub use types::{JsBuffer, JsArrayBuffer, BinaryData, JsError, Value, JsValue, JsUndefined, JsNull, JsBoolean, JsString, JsNumber, JsObject, JsArray, JsFunction};
pub use object::{Object, Class};
pub use borrow::{Borrow, BorrowMut};
pub use context::{CallKind, Context, ModuleContext, ExecuteContext, ComputeContext, CallContext, FunctionContext, MethodContext, TaskContext};
pub use result::{NeonResult, JsResult, JsResultExt};
pub use task::Task;
#[cfg(feature = "event-handler-api")]
pub use event::EventHandler;
pub use crate::{register_module, declare_types};
#[cfg(feature = "napi-1")]
pub use crate::{
    handle::Root,
    types::boxed::{Finalize, JsBox}
};
#[cfg(feature = "napi-4")]
pub use crate::task::{EventQueue, EventQueueError};
