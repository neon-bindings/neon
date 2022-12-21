use std::{error, fmt};

use serde::{de as der, ser, Serialize};

use crate::{
    context::Context,
    handle::Handle,
    result::{JsResult, NeonResult, ResultExt, Throw},
    types::{JsValue, Value},
};

mod de;
mod se;
mod sys;

/// Attempts to read a JavaScript value into a Rust data type using the serde::Deserialize implementation
pub fn deserialize<'cx, T, V, C>(cx: &mut C, v: Handle<V>) -> NeonResult<T>
where
    T: der::DeserializeOwned + ?Sized,
    V: Value,
    C: Context<'cx>,
{
    unsafe { T::deserialize(de::Deserializer::new(cx.env().to_raw(), v.to_raw())).or_throw(cx) }
}

/// Attempts to write Rust data into a JavaScript value using the serde::Serialize implementation
pub fn serialize<'cx, T, V, C>(cx: &mut C, v: &V) -> JsResult<'cx, T>
where
    T: Value,
    V: Serialize + ?Sized,
    C: Context<'cx>,
{
    let v = unsafe {
        v.serialize(se::Serializer::new(cx.env().to_raw()))
            .or_throw(cx)?
    };

    JsValue::new_internal(v).downcast_or_throw(cx)
}

#[derive(Clone, Debug, PartialEq)]
/// This type represents all possible errors that can occur when serializing or
/// deserializing JavaScript types.
struct Error {
    kind: ErrorKind,
}

impl error::Error for Error {}

impl Error {
    fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    /// Indicates if the error was due to an exception in the JavaScript VM
    /// If an exception is pending, all other JavaScript operations will fail
    /// until it is cleared.
    pub fn is_exception_pending(&self) -> bool {
        self.kind == ErrorKind::NodeApi(sys::Status::PendingException)
    }

    fn is_array_expected(&self) -> bool {
        self.kind == ErrorKind::NodeApi(sys::Status::ArrayExpected)
    }

    fn missing_key() -> Self {
        ErrorKind::MissingKey.into()
    }

    fn unsupported_type(typ: sys::ValueType) -> Self {
        ErrorKind::UnsupportedType(typ).into()
    }

    fn unsupported_key_type(typ: &'static str) -> Self {
        ErrorKind::UnsupportedKeyType(typ).into()
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ErrorKind {
    // Serde codec errors
    Custom(String),

    // Attempted to use a key type that is not supported by JavaScript
    UnsupportedKeyType(&'static str),

    // Serde reads and writes key/value pairs as distinct steps requiring
    // Neon to cache the intermediate key. This error is unexpected and should
    // never occur outside of a buggy serde implementation.
    MissingKey,

    // deserialize_any
    UnsupportedType(sys::ValueType),

    // N-API
    NodeApi(sys::Status),
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error::new(kind)
    }
}

impl From<sys::Status> for Error {
    fn from(other: sys::Status) -> Self {
        ErrorKind::NodeApi(other).into()
    }
}

impl der::Error for Error {
    fn custom<T: fmt::Display>(err: T) -> Self {
        Error {
            kind: ErrorKind::Custom(err.to_string()),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(err: T) -> Self {
        der::Error::custom(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Custom(err) => f.write_str(err),
            ErrorKind::UnsupportedKeyType(ty) => write!(f, "UnsupportedKeyType({})", ty),
            ErrorKind::MissingKey => f.write_str("MissingKey"),
            ErrorKind::UnsupportedType(typ) => write!(f, "UnsupportedType({:?})", typ),
            ErrorKind::NodeApi(err) => write!(f, "Node-API ({:?})", err),
        }
    }
}

impl<T> ResultExt<T> for Result<T, Error> {
    fn or_throw<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) if e.is_exception_pending() => Err(Throw::new()),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}
