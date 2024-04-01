use crate::{
    context::Context,
    handle::Handle,
    object::Object,
    result::{JsResult, NeonResult},
    types::{
        extract::{private, TryFromJs},
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

/// Extract a value by serializing to JSON
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

impl<T> private::Sealed for Json<T> {}
