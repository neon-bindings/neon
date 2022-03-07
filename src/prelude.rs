//! Convenience module for the most common Neon imports.

#[cfg(feature = "legacy-runtime")]
#[doc(no_inline)]
pub use crate::borrow::{Borrow, BorrowMut};
#[doc(no_inline)]
pub use crate::context::{
    CallContext, CallKind, ComputeContext, Context, ExecuteContext, FunctionContext, MethodContext,
    ModuleContext, TaskContext,
};
#[cfg(feature = "legacy-runtime")]
#[doc(no_inline)]
pub use crate::declare_types;
#[cfg(all(not(feature = "napi-1"), feature = "event-handler-api"))]
#[doc(no_inline)]
pub use crate::event::EventHandler;
#[cfg(feature = "napi-4")]
#[doc(no_inline)]
pub use crate::event::{Channel, SendError};
#[doc(no_inline)]
pub use crate::handle::Handle;
#[cfg(feature = "legacy-runtime")]
#[doc(no_inline)]
pub use crate::object::Class;
#[doc(no_inline)]
pub use crate::object::Object;
#[doc(no_inline)]
pub use crate::register_module;
#[doc(no_inline)]
pub use crate::result::{JsResult, JsResultExt, NeonResult, ResultExt as NeonResultExt};
#[cfg(feature = "legacy-runtime")]
pub use crate::task::Task;
#[cfg(feature = "legacy-runtime")]
#[doc(no_inline)]
pub use crate::types::BinaryData;
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
