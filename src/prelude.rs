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
pub use crate::borrow::{Borrow, BorrowMut};
pub use crate::context::{
    CallContext, CallKind, ComputeContext, Context, ExecuteContext, FunctionContext, MethodContext,
    ModuleContext, TaskContext,
};
#[cfg(all(not(feature = "napi-1"), feature = "event-handler-api"))]
pub use crate::event::EventHandler;
pub use crate::handle::Handle;
#[cfg(feature = "legacy-runtime")]
pub use crate::object::Class;
pub use crate::object::Object;
pub use crate::result::{JsResult, JsResultExt, NeonResult};
#[cfg(feature = "legacy-runtime")]
pub use crate::task::Task;
pub use crate::types::{
    BinaryData, JsArray, JsArrayBuffer, JsBoolean, JsBuffer, JsError, JsFunction, JsNull, JsNumber,
    JsObject, JsString, JsUndefined, JsValue, Value,
};
