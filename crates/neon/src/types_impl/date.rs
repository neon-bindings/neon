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

/// A JavaScript Date object
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

/// The Error struct for a Date
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
    Overflow,
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
    /// The smallest possible Date value, defined by ECMAScript. See <https://www.ecma-international.org/ecma-262/5.1/#sec-15.7.3.3>
    pub const MIN_VALUE: f64 = -8.64e15;
    /// The largest possible Date value, defined by ECMAScript. See <https://www.ecma-international.org/ecma-262/5.1/#sec-15.7.3.2>
    pub const MAX_VALUE: f64 = 8.64e15;

    /// Creates a new Date. It errors when `value` is outside the range of valid JavaScript Date values. When `value`
    /// is `NaN`, the operation will succeed but with an invalid Date
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

    /// Creates a new Date with lossy conversion for out of bounds Date values. Out of bounds
    /// values will be treated as NaN
    pub fn new_lossy<'a, C: Context<'a>, V: Into<f64>>(cx: &mut C, value: V) -> Handle<'a, JsDate> {
        let env = cx.env().to_raw();
        let local = unsafe { sys::date::new_date(env, value.into()) };
        Handle::new_internal(JsDate(local))
    }

    /// Gets the Date's value. An invalid Date will return `std::f64::NaN`
    pub fn value<'a, C: Context<'a>>(&self, cx: &mut C) -> f64 {
        let env = cx.env().to_raw();
        unsafe { sys::date::value(env, self.to_raw()) }
    }

    /// Checks if the Date's value is valid. A Date is valid if its value is between
    /// `JsDate::MIN_VALUE` and `JsDate::MAX_VALUE` or if it is `NaN`
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
