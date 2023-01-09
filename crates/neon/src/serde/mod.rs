use std::{error, fmt};

use crate::{
    context::Context,
    handle::Handle,
    object::Object,
    result::{JsResult, NeonResult, Throw},
    types::{JsFunction, JsObject, JsValue, Value},
};

#[cfg(feature = "napi-6")]
use crate::{handle::Root, thread::LocalKey};

pub(crate) use args::{FromArg, FromArgs};

mod args;
mod de;
mod ser;
mod sys;

#[derive(Debug)]
pub(super) enum Error {
    Custom(String),
    Unsupported(sys::ValueType),
    Status(sys::Status),
    FallbackJson,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Custom(msg) => f.write_str(msg),
            Error::Unsupported(typ) => write!(f, "Unsupported({:?})", typ),
            Error::Status(status) => write!(f, "Status({:?})", status),
            Error::FallbackJson => f.write_str("FallbackJson"),
        }
    }
}

impl error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

impl From<sys::Status> for Error {
    fn from(status: sys::Status) -> Self {
        Self::Status(status)
    }
}

impl From<Throw> for Error {
    fn from(_: Throw) -> Self {
        Self::Status(sys::Status::PendingException)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Custom(err.to_string())
    }
}

fn parse<'cx, C: Context<'cx>>(cx: &mut C) -> JsResult<'cx, JsFunction> {
    fn parse<'cx, C: Context<'cx>>(cx: &mut C) -> JsResult<'cx, JsFunction> {
        cx.global()
            .get::<JsObject, _, _>(cx, "JSON")?
            .get::<JsFunction, _, _>(cx, "parse")
    }

    #[cfg(feature = "napi-6")]
    {
        static PARSE: LocalKey<Root<JsFunction>> = LocalKey::new();

        PARSE
            .get_or_try_init(cx, |cx| Ok(parse(cx)?.root(cx)))
            .map(|v| v.to_inner(cx))
    }

    #[cfg(not(feature = "napi-6"))]
    {
        parse(cx)
    }
}

fn stringify<'cx, C: Context<'cx>>(cx: &mut C) -> JsResult<'cx, JsFunction> {
    fn stringify<'cx, C: Context<'cx>>(cx: &mut C) -> JsResult<'cx, JsFunction> {
        cx.global()
            .get::<JsObject, _, _>(cx, "JSON")?
            .get::<JsFunction, _, _>(cx, "stringify")
    }

    #[cfg(feature = "napi-6")]
    {
        static STRINGIFY: LocalKey<Root<JsFunction>> = LocalKey::new();

        STRINGIFY
            .get_or_try_init(cx, |cx| Ok(stringify(cx)?.root(cx)))
            .map(|v| v.to_inner(cx))
    }

    #[cfg(not(feature = "napi-6"))]
    {
        stringify(cx)
    }
}

/// Attempts to read a JavaScript value into a Rust data type using serde
pub fn deserialize<'cx, T, V, C>(cx: &mut C, v: Handle<V>) -> NeonResult<T>
where
    T: serde::de::DeserializeOwned + ?Sized,
    V: Value,
    C: Context<'cx>,
{
    de::deserialize(cx, v).or_else(|err| cx.throw_error(dbg!(err).to_string()))
}

/// Attempts to write Rust data into a JavaScript value using serde
pub fn serialize<'cx, T, V, C>(cx: &mut C, v: &V) -> JsResult<'cx, T>
where
    T: Value,
    V: serde::ser::Serialize + ?Sized,
    C: Context<'cx>,
{
    let v = match v.serialize(unsafe { ser::Serializer::new(cx.env().to_raw()) }) {
        Ok(v) => JsValue::new_internal(v),
        Err(Error::FallbackJson) => {
            let s = serde_json::to_string(v).or_else(|err| cx.throw_error(err.to_string()))?;
            let s = cx.string(s);
            let this = cx.undefined();

            parse(cx)?.call(cx, this, [s.upcast()])?
        }
        Err(err) => return cx.throw_error(err.to_string()),
    };

    v.downcast_or_throw(cx)
}
