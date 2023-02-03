use std::{
    error::Error,
    fmt::{self, Debug},
};

use super::{private::ValueInternal, Value};

use crate::{
    context::{internal::Env, Context},
    handle::{internal::TransparentNoCopyWrapper, Handle, Managed},
    object::Object,
    result::{JsResult, ResultExt},
    sys::{self, raw},
};

/// The type of JavaScript
/// [`Date`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date)
/// objects.
///
/// # Example
///
/// The following shows an example of converting Rust
/// [`SystemTime`](std::time::SystemTime) timestamps to JavaScript `Date` objects.
///
/// ```
/// # use neon::prelude::*;
/// use easy_cast::Cast; // for safe numeric conversions
/// use neon::types::JsDate;
/// use std::{error::Error, fs::File, time::SystemTime};
///
/// /// Return the "modified" timestamp for the file at the given path.
/// fn last_modified(path: &str) -> Result<f64, Box<dyn Error>> {
///     Ok(File::open(&path)?
///         .metadata()?
///         .modified()?
///         .duration_since(SystemTime::UNIX_EPOCH)?
///         .as_millis()
///         .try_cast()?)
/// }
///
/// fn modified(mut cx: FunctionContext) -> JsResult<JsDate> {
///     let path: Handle<JsString> = cx.argument(0)?;
///
///     last_modified(&path.value(&mut cx))
///         .and_then(|n| Ok(cx.date(n)?))
///         .or_else(|err| cx.throw_error(err.to_string()))
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "napi-5")))]
#[derive(Debug)]
#[repr(transparent)]
pub struct JsDate(raw::Local);

impl Value for JsDate {}

unsafe impl TransparentNoCopyWrapper for JsDate {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl Managed for JsDate {
    fn to_raw(&self) -> raw::Local {
        self.0
    }

    fn from_raw(_: Env, h: raw::Local) -> Self {
        JsDate(h)
    }
}

/// An error produced when constructing a date with an invalid value.
#[derive(Debug)]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-5")))]
pub struct DateError(DateErrorKind);

impl DateError {
    pub fn kind(&self) -> DateErrorKind {
        self.0
    }
}

impl fmt::Display for DateError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.0.as_str())
    }
}

impl Error for DateError {}

/// The error kinds corresponding to `DateError`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-5")))]
pub enum DateErrorKind {
    /// Produced for an initialization value greater than
    /// [`JsDate::MAX_VALUE`](JsDate::MAX_VALUE).
    Overflow,
    /// Produced for an initialization value lesser than
    /// [`JsDate::MIN_VALUE`](JsDate::MIN_VALUE).
    Underflow,
}

impl DateErrorKind {
    fn as_str(&self) -> &'static str {
        match *self {
            DateErrorKind::Overflow => "Date overflow",
            DateErrorKind::Underflow => "Date underflow",
        }
    }
}

impl<'a, T: Value> ResultExt<Handle<'a, T>> for Result<Handle<'a, T>, DateError> {
    /// Creates an `Error` on error
    fn or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> JsResult<'a, T> {
        self.or_else(|e| cx.throw_range_error(e.0.as_str()))
    }
}

impl JsDate {
    /// The smallest possible `Date` value,
    /// [defined by ECMAScript](https://www.ecma-international.org/ecma-262/5.1/#sec-15.7.3.3).
    pub const MIN_VALUE: f64 = -8.64e15;
    /// The largest possible `Date` value,
    /// [defined by ECMAScript](https://www.ecma-international.org/ecma-262/5.1/#sec-15.7.3.2).
    pub const MAX_VALUE: f64 = 8.64e15;

    /// Creates a new `Date`. It errors when `value` is outside the range of valid JavaScript
    /// `Date` values. When `value` is `NaN`, the operation will succeed but with an
    /// invalid `Date`.
    pub fn new<'a, C: Context<'a>, T: Into<f64>>(
        cx: &mut C,
        value: T,
    ) -> Result<Handle<'a, JsDate>, DateError> {
        let env = cx.env().to_raw();
        let time = value.into();

        if time > JsDate::MAX_VALUE {
            return Err(DateError(DateErrorKind::Overflow));
        } else if time < JsDate::MIN_VALUE {
            return Err(DateError(DateErrorKind::Underflow));
        }

        let local = unsafe { sys::date::new_date(env, time) };
        let date = Handle::new_internal(JsDate(local));
        Ok(date)
    }

    /// Creates a new `Date` with lossy conversion for out of bounds `Date` values.
    /// Out of bounds values will be treated as `NaN`.
    pub fn new_lossy<'a, C: Context<'a>, V: Into<f64>>(cx: &mut C, value: V) -> Handle<'a, JsDate> {
        let env = cx.env().to_raw();
        let local = unsafe { sys::date::new_date(env, value.into()) };
        Handle::new_internal(JsDate(local))
    }

    /// Gets the `Date`'s value. An invalid `Date` will return [`std::f64::NAN`].
    pub fn value<'a, C: Context<'a>>(&self, cx: &mut C) -> f64 {
        let env = cx.env().to_raw();
        unsafe { sys::date::value(env, self.to_raw()) }
    }

    /// Checks if the `Date`'s value is valid. A `Date` is valid if its value is
    /// between [`JsDate::MIN_VALUE`] and [`JsDate::MAX_VALUE`] or if it is `NaN`.
    pub fn is_valid<'a, C: Context<'a>>(&self, cx: &mut C) -> bool {
        let value = self.value(cx);
        (JsDate::MIN_VALUE..=JsDate::MAX_VALUE).contains(&value)
    }
}

impl ValueInternal for JsDate {
    fn name() -> String {
        "object".to_string()
    }

    fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool {
        unsafe { sys::tag::is_date(env.to_raw(), other.to_raw()) }
    }
}

impl Object for JsDate {}
