use crate::{
    context::Context,
    handle::Handle,
    object::Object,
    result::{JsResult, NeonResult},
    types::{
        extract::{private, TryFromJs, TryIntoJs},
        JsFunction, JsObject, JsString, JsValue,
    },
};

#[cfg(feature = "napi-6")]
use crate::{handle::Root, thread::LocalKey};

fn global_json_stringify<'cx, C>(cx: &mut C) -> JsResult<'cx, JsFunction>
where
    C: Context<'cx>,
{
    cx.global::<JsObject>("JSON")?.get(cx, "stringify")
}

#[cfg(not(feature = "napi-6"))]
// N.B.: This is not semantically identical to Node-API >= 6. Patching the global
// method could cause differences between calls. However, threading a `Root` through
// would require a significant refactor and "don't do this or things will break" is
// fairly common in JS.
fn json_stringify<'cx, C>(cx: &mut C) -> JsResult<'cx, JsFunction>
where
    C: Context<'cx>,
{
    global_json_stringify(cx)
}

#[cfg(feature = "napi-6")]
fn json_stringify<'cx, C>(cx: &mut C) -> JsResult<'cx, JsFunction>
where
    C: Context<'cx>,
{
    static STRINGIFY: LocalKey<Root<JsFunction>> = LocalKey::new();

    STRINGIFY
        .get_or_try_init(cx, |cx| global_json_stringify(cx).map(|f| f.root(cx)))
        .map(|f| f.to_inner(cx))
}

fn stringify<'cx, C>(cx: &mut C, v: Handle<JsValue>) -> NeonResult<String>
where
    C: Context<'cx>,
{
    json_stringify(cx)?
        .call(cx, v, [v])?
        .downcast_or_throw::<JsString, _>(cx)
        .map(|s| s.value(cx))
}

fn global_json_parse<'cx, C>(cx: &mut C) -> JsResult<'cx, JsFunction>
where
    C: Context<'cx>,
{
    cx.global::<JsObject>("JSON")?.get(cx, "parse")
}

#[cfg(not(feature = "napi-6"))]
fn json_parse<'cx, C>(cx: &mut C) -> JsResult<'cx, JsFunction>
where
    C: Context<'cx>,
{
    global_json_parse(cx)
}

#[cfg(feature = "napi-6")]
fn json_parse<'cx, C>(cx: &mut C) -> JsResult<'cx, JsFunction>
where
    C: Context<'cx>,
{
    static PARSE: LocalKey<Root<JsFunction>> = LocalKey::new();

    PARSE
        .get_or_try_init(cx, |cx| global_json_parse(cx).map(|f| f.root(cx)))
        .map(|f| f.to_inner(cx))
}

fn parse<'cx, C>(cx: &mut C, s: &str) -> JsResult<'cx, JsValue>
where
    C: Context<'cx>,
{
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
    type Error = serde_json::Error;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        Ok(serde_json::from_str(&stringify(cx, v)?).map(Json))
    }

    fn from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        Self::try_from_js(cx, v)?.or_else(|err| cx.throw_error(err.to_string()))
    }
}

impl<'cx, T> TryIntoJs<'cx> for Json<T>
where
    T: serde::Serialize,
{
    type Value = JsValue;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        let s = serde_json::to_string(&self.0).or_else(|err| cx.throw_error(err.to_string()))?;

        parse(cx, &s)
    }
}

impl<T> private::Sealed for Json<T> {}
