use std::{error, fmt};

use crate::{
    context::Context,
    handle::Handle,
    object::Object,
    result::{JsResult, NeonResult, ResultExt},
    types::{
        extract::{TryFromJs, TryIntoJs, TypeExpected},
        JsError, JsString, JsValue,
    },
};

type BoxError = Box<dyn error::Error + Send + Sync + 'static>;

#[derive(Debug)]
/// Error that implements `TryIntoJs` and can produce specific error types
pub struct Error {
    cause: BoxError,
    kind: Option<ErrorKind>,
}

#[derive(Debug)]
enum ErrorKind {
    Error,
    RangeError,
    TypeError,
}

impl Error {
    pub fn new<E>(cause: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self::create(ErrorKind::Error, cause)
    }

    pub fn range_error<E>(cause: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self::create(ErrorKind::RangeError, cause)
    }

    pub fn type_error<E>(cause: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self::create(ErrorKind::TypeError, cause)
    }

    pub fn is_error(&self) -> bool {
        matches!(self.kind, Some(ErrorKind::Error))
    }

    pub fn is_range_error(&self) -> bool {
        matches!(self.kind, Some(ErrorKind::RangeError))
    }

    pub fn is_type_error(&self) -> bool {
        matches!(self.kind, Some(ErrorKind::TypeError))
    }

    pub fn cause(&self) -> &BoxError {
        &self.cause
    }

    pub fn into_cause(self) -> BoxError {
        self.cause
    }

    fn create<E>(kind: ErrorKind, cause: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self {
            cause: cause.into(),
            kind: Some(kind),
        }
    }
}

impl<E> From<E> for Error
where
    E: Into<BoxError>,
{
    fn from(cause: E) -> Self {
        Self::new(cause)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.cause)
    }
}

impl<'cx> TryIntoJs<'cx> for Error {
    type Value = JsError;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        let message = self.cause.to_string();

        match self.kind {
            Some(ErrorKind::RangeError) => cx.range_error(message),
            Some(ErrorKind::TypeError) => cx.type_error(message),
            _ => cx.error(message),
        }
    }
}

impl<'cx> TryFromJs<'cx> for Error {
    type Error = TypeExpected<JsError>;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        let err = match v.downcast::<JsError, _>(cx) {
            Ok(err) => err,
            Err(_) => return Ok(Err(Self::Error::new())),
        };

        Ok(Ok(Self {
            cause: get_message(cx, err)?.into(),
            kind: get_kind(cx, err)?,
        }))
    }

    fn from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        Self::try_from_js(cx, v)?.or_throw(cx)
    }
}

fn get_message<'cx, C>(cx: &mut C, err: Handle<JsError>) -> NeonResult<String>
where
    C: Context<'cx>,
{
    let message = err
        .get_value(cx, "message")?
        .downcast::<JsString, _>(cx)
        .map(|v| v.value(cx))
        .unwrap_or_default();

    Ok(message)
}

fn get_kind<'cx, C>(cx: &mut C, err: Handle<JsError>) -> NeonResult<Option<ErrorKind>>
where
    C: Context<'cx>,
{
    let name = match err.get_value(cx, "name")?.downcast::<JsString, _>(cx) {
        Ok(v) => v.value(cx),
        Err(_) => return Ok(None),
    };

    Ok(Some(match name.as_str() {
        "TypeError" => ErrorKind::TypeError,
        "RangeError" => ErrorKind::RangeError,
        _ => return Ok(None),
    }))
}
