use std::{error, fmt};

use crate::{
    context::{Context, Cx},
    result::JsResult,
    types::{extract::TryIntoJs, JsError},
};

type BoxError = Box<dyn error::Error + Send + Sync + 'static>;

#[derive(Debug)]
/// Error that implements [`TryIntoJs`] and can produce specific error types.
///
/// [`Error`] implements [`From`] for most error types, allowing ergonomic error handling in
/// exported functions with the `?` operator.
///
/// ### Example
///
/// ```
/// use neon::types::extract::Error;
///
/// #[neon::export]
/// fn read_file(path: String) -> Result<String, Error> {
///     let contents = std::fs::read_to_string(path)?;
///     Ok(contents)
/// }
/// ```
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
    /// Create a new [`Error`] from a `cause`
    pub fn new<E>(cause: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self::create(ErrorKind::Error, cause)
    }

    /// Create a `RangeError`
    pub fn range_error<E>(cause: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self::create(ErrorKind::RangeError, cause)
    }

    /// Create a `TypeError`
    pub fn type_error<E>(cause: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self::create(ErrorKind::TypeError, cause)
    }

    /// Check if error is a `RangeError`
    pub fn is_range_error(&self) -> bool {
        matches!(self.kind, Some(ErrorKind::RangeError))
    }

    /// Check if error is a `TypeError`
    pub fn is_type_error(&self) -> bool {
        matches!(self.kind, Some(ErrorKind::TypeError))
    }

    /// Get a reference to the underlying `cause`
    pub fn cause(&self) -> &BoxError {
        &self.cause
    }

    /// Extract the `std::error::Error` cause
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

// Blanket impl allow for ergonomic `?` error handling from typical error types (including `anyhow`)
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

// N.B.: `TryFromJs` is not included. If Neon were to add support for additional error types,
// this would be a *breaking* change. We will wait for user demand before providing this feature.
impl<'cx> TryIntoJs<'cx> for Error {
    type Value = JsError;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        let message = self.cause.to_string();

        match self.kind {
            Some(ErrorKind::RangeError) => cx.range_error(message),
            Some(ErrorKind::TypeError) => cx.type_error(message),
            _ => cx.error(message),
        }
    }
}
