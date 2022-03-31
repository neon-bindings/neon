//! Convenience module for the most common Neon imports.

#[doc(no_inline)]
pub use crate::context::{
    CallContext, CallKind, ComputeContext, Context, ExecuteContext, FunctionContext, MethodContext,
    ModuleContext, TaskContext,
};
#[cfg(all(not(feature = "napi-1"), feature = "event-handler-api"))]
#[doc(no_inline)]
pub use crate::event::EventHandler;
#[cfg(feature = "napi-4")]
#[doc(no_inline)]
pub use crate::event::{Channel, SendError};
#[cfg(feature = "napi-4")]
#[doc(no_inline)]
#[allow(deprecated)]
pub use crate::event::{EventQueue, EventQueueError};
#[doc(no_inline)]
pub use crate::handle::Handle;
#[doc(no_inline)]
pub use crate::object::Object;
#[doc(no_inline)]
pub use crate::result::{JsResult, JsResultExt, NeonResult, ResultExt as NeonResultExt};
#[cfg(feature = "napi-1")]
#[doc(no_inline)]
pub use crate::types::JsPromise;
#[cfg(feature = "napi-1")]
#[doc(no_inline)]
pub use crate::types::JsTypedArray;
#[doc(no_inline)]
pub use crate::types::{
    JsArray, JsArrayBuffer, JsBoolean, JsBuffer, JsError, JsFunction, JsNull, JsNumber, JsObject,
    JsString, JsUndefined, JsValue, Value,
};
#[cfg(feature = "napi-1")]
#[doc(no_inline)]
pub use crate::{
    handle::Root,
    types::boxed::{Finalize, JsBox},
};
