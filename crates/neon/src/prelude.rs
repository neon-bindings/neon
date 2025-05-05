//! Convenience module for the most common Neon imports.

#[doc(no_inline)]
pub use crate::{
    context::{CallKind, Context, Cx, FunctionContext, ModuleContext},
    handle::{Handle, Root},
    object::Object,
    result::{JsResult, NeonResult, ResultExt as NeonResultExt},
    types::{
        JsArray, JsArrayBuffer, JsBigInt64Array, JsBigUint64Array, JsBoolean, JsBuffer, JsError,
        JsFloat32Array, JsFloat64Array, JsFunction, JsInt8Array, JsInt16Array, JsInt32Array,
        JsNull, JsNumber, JsObject, JsPromise, JsString, JsTypedArray, JsUint8Array, JsUint16Array,
        JsUint32Array, JsUndefined, JsValue, Value,
        boxed::{Finalize, JsBox},
    },
};

#[doc(hidden)]
pub use crate::context::{ComputeContext, ExecuteContext, TaskContext};

#[cfg(feature = "napi-4")]
#[doc(no_inline)]
pub use crate::event::{Channel, SendError};

#[cfg(feature = "napi-4")]
#[doc(no_inline)]
#[allow(deprecated)]
pub use crate::event::{EventQueue, EventQueueError};
