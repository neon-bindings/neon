//! A convenience module that re-exports the most commonly-used Neon APIs.

pub use handle::Handle;
pub use types::{JsBuffer, JsArrayBuffer, BinaryData, JsError, Value, JsValue, JsUndefined, JsNull, JsBoolean, JsString, JsNumber, JsObject, JsArray, JsFunction};
pub use object::Object;
#[cfg(feature = "legacy-runtime")]
pub use object::Class;
pub use borrow::{Borrow, BorrowMut};
pub use context::{CallKind, Context, ModuleContext, ExecuteContext, ComputeContext, CallContext, FunctionContext, MethodContext, TaskContext};
pub use result::{NeonResult, JsResult, JsResultExt};
#[cfg(feature = "legacy-runtime")]
pub use task::Task;
#[cfg(all(not(feature = "napi-1"), feature = "event-handler-api"))]
pub use event::EventHandler;
pub use crate::register_module;
#[cfg(feature = "legacy-runtime")]
pub use crate::declare_types;
#[cfg(feature = "napi-1")]
pub use crate::{
    handle::Root,
    types::boxed::{Finalize, JsBox}
};
#[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
pub use crate::event::{EventQueue, EventQueueError};
