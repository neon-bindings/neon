//! Convenience module for the most common Neon imports.

#[doc(no_inline)]
pub use crate::{
    context::{
        CallContext, CallKind, ComputeContext, Context, ExecuteContext, FunctionContext,
        MethodContext, ModuleContext, TaskContext,
    },
    handle::{Handle, Root},
    object::Object,
    result::{JsResult, JsResultExt, NeonResult, ResultExt as NeonResultExt},
    types::{
        boxed::{Finalize, JsBox},
        JsArray, JsArrayBuffer, JsBoolean, JsBuffer, JsError, JsFunction, JsNull, JsNumber,
        JsObject, JsPromise, JsString, JsTypedArray, JsUndefined, JsValue, Value,
    },
};

#[cfg(feature = "napi-4")]
#[doc(no_inline)]
pub use crate::event::{Channel, SendError};

#[cfg(feature = "napi-4")]
#[doc(no_inline)]
#[allow(deprecated)]
pub use crate::event::{EventQueue, EventQueueError};
