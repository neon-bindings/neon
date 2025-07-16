//! Extract JavaScript values with JSON serialization
//!
//! For complex objects that implement [`serde::Serialize`] and [`serde::Deserialize`],
//! it is more ergonomic--and often faster--to extract with JSON serialization. The [`Json`]
//! extractor automatically calls `JSON.stringify` and `JSON.parse` as necessary.
//!
//! ```
//! use neon::types::extract::Json;
//!
//! #[neon::export]
//! fn sort(Json(mut strings): Json<Vec<String>>) -> Json<Vec<String>> {
//!     strings.sort();
//!     Json(strings)
//! }
//! ```

use std::{error, fmt};

use crate::{
    context::{Context, Cx},
    handle::Handle,
    object::Object,
    result::{JsResult, NeonResult},
    types::{
        extract::{private, TryFromJs, TryIntoJs},
        JsError, JsFunction, JsObject, JsString, JsValue,
    },
};

#[cfg(feature = "napi-6")]
use crate::{handle::Root, thread::LocalKey};

fn global_json_stringify<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction> {
    cx.global::<JsObject>("JSON")?.get(cx, "stringify")
}

#[cfg(not(feature = "napi-6"))]
// N.B.: This is not semantically identical to Node-API >= 6. Patching the global
// method could cause differences between calls. However, threading a `Root` through
// would require a significant refactor and "don't do this or things will break" is
// fairly common in JS.
fn json_stringify<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction> {
    global_json_stringify(cx)
}

#[cfg(feature = "napi-6")]
fn json_stringify<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction> {
    static STRINGIFY: LocalKey<Root<JsFunction>> = LocalKey::new();

    STRINGIFY
        .get_or_try_init(cx, |cx| global_json_stringify(cx).map(|f| f.root(cx)))
        .map(|f| f.to_inner(cx))
}

fn stringify(cx: &mut Cx, v: Handle<JsValue>) -> NeonResult<String> {
    json_stringify(cx)?
        .call(cx, v, [v])?
        .downcast_or_throw::<JsString, _>(cx)
        .map(|s| s.value(cx))
}

fn global_json_parse<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction> {
    cx.global::<JsObject>("JSON")?.get(cx, "parse")
}

#[cfg(not(feature = "napi-6"))]
fn json_parse<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction> {
    global_json_parse(cx)
}

#[cfg(feature = "napi-6")]
fn json_parse<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction> {
    static PARSE: LocalKey<Root<JsFunction>> = LocalKey::new();

    PARSE
        .get_or_try_init(cx, |cx| global_json_parse(cx).map(|f| f.root(cx)))
        .map(|f| f.to_inner(cx))
}

fn parse<'cx>(cx: &mut Cx<'cx>, s: &str) -> JsResult<'cx, JsValue> {
    let s = cx.string(s).upcast();

    json_parse(cx)?.call(cx, s, [s])
}

/// Wrapper for converting between `T` and [`JsValue`](crate::types::JsValue) by
/// serializing with JSON.
pub struct Json<T>(pub T);

impl<'cx, T> TryFromJs<'cx> for Json<T>
where
    for<'de> T: serde::de::Deserialize<'de>,
{
    type Error = Error;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        Ok(serde_json::from_str(&stringify(cx, v)?)
            .map(Json)
            .map_err(Error))
    }
}

impl<'cx, T> TryIntoJs<'cx> for Json<T>
where
    T: serde::Serialize,
{
    type Value = JsValue;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        TryIntoJs::try_into_js(&self, cx)
    }
}

impl<'cx, T> TryIntoJs<'cx> for &Json<T>
where
    T: serde::Serialize,
{
    type Value = JsValue;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        let s = serde_json::to_string(&self.0).or_else(|err| cx.throw_error(err.to_string()))?;

        parse(cx, &s)
    }
}

impl<T> private::Sealed for Json<T> {}

impl<T> private::Sealed for &Json<T> {}

/// Error returned when a value is invalid JSON
pub struct Error(serde_json::Error);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl error::Error for Error {}

impl<'cx> TryIntoJs<'cx> for Error {
    type Value = JsError;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsError::error(cx, self.to_string())
    }
}

impl private::Sealed for Error {}
