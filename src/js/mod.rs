//! Types and traits representing JavaScript values.

pub mod binary;
pub mod error;

pub use internal::js::{Value, Variant, Object, JsValue, JsUndefined, JsNull, JsBoolean, JsInteger, JsNumber, JsString, JsObject, JsArray, JsFunction};
