//! Convenience module for the most common Neon imports.

#[cfg(feature = "legacy-runtime")]
pub use crate::declare_types;
#[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
pub use crate::event::{EventQueue, EventQueueError};
pub use crate::register_module;
#[cfg(feature = "napi-1")]
pub use crate::{
    handle::Root,
    types::boxed::{Finalize, JsBox},
};
pub use borrow::{Borrow, BorrowMut};
pub use context::{
    CallContext, CallKind, ComputeContext, Context, ExecuteContext, FunctionContext, MethodContext,
    ModuleContext, TaskContext,
};
#[cfg(all(not(feature = "napi-1"), feature = "event-handler-api"))]
pub use event::EventHandler;
pub use handle::Handle;
#[cfg(feature = "legacy-runtime")]
pub use object::Class;
pub use object::Object;
pub use result::{JsResult, JsResultExt, NeonResult};
#[cfg(feature = "legacy-runtime")]
pub use task::Task;
pub use types::{
    BinaryData, JsArray, JsArrayBuffer, JsBoolean, JsBuffer, JsError, JsFunction, JsNull, JsNumber,
    JsObject, JsString, JsUndefined, JsValue, Value,
};
