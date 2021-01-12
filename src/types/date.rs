use std::fmt;
use std::fmt::Debug;
use std::error::Error;
use neon_runtime;
use neon_runtime::raw;
use context::{Context};
use context::internal::Env;
use result::{JsResult, JsResultExt};
use object::{Object};
use handle::{Handle, Managed};
use super::{Value, ValueInternal};

/// A JavaScript Date object
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JsDate(raw::Local);

impl Value for JsDate { }

impl Managed for JsDate {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(_: Env, h: raw::Local) -> Self { JsDate(h) }
}

/// The Error struct for a Date
#[derive(Debug)]
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
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DateErrorKind {
    Overflow,
    Underflow,
}

impl DateErrorKind {
    fn as_str(&self) -> &'static str {
        match *self {
            DateErrorKind::Overflow => "Uncaught RangeError: Date overflow",
            DateErrorKind::Underflow => "Uncaught RangeError: Date underflow",
        }
    }
}

impl<'a, T: Value> JsResultExt<'a, T> for Result<Handle<'a, T>, DateError> {
    /// Creates an `Error` on error
    fn or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> JsResult<'a, T> {
        self.or_else(|e| cx.throw_range_error(e.0.as_str()))
    }
}

impl JsDate {
    /// The smallest possible Date value, defined by ECMAScript. See https://www.ecma-international.org/ecma-262/5.1/#sec-15.7.3.3
    pub const MIN_VALUE: f64 = -8.64e15;
    /// The largest possible Date value, defined by ECMAScript. See https://www.ecma-international.org/ecma-262/5.1/#sec-15.7.3.2
    pub const MAX_VALUE: f64 = 8.64e15;

    /// Create a new Date. It errors when `value` is an out of bounds JavaScript Date value. When `value`
    /// is `NaN`, an invalid
    pub fn new<'a, C: Context<'a>, T: Into<f64>>(cx: &mut C, value: T) -> Result<Handle<'a, JsDate>, DateError> {
        let env = cx.env().to_raw();
        let time = value.into();

        if time > JsDate::MAX_VALUE {
            return Err(DateError(DateErrorKind::Overflow))
        } else if time < JsDate::MIN_VALUE {
            return Err(DateError(DateErrorKind::Underflow))
        }

        let local = unsafe {
            neon_runtime::date::new_date(env, time)
        };
        let date = Handle::new_internal(JsDate(local));
        Ok(date)
    }

    /// Create a new Date with lossy conversion for out of bounds Date values.
    pub fn new_lossy<'a, C: Context<'a>, V: Into<f64>>(cx: &mut C, value: V) -> Handle<'a, JsDate> {
        let env = cx.env().to_raw();
        let local = unsafe {
            neon_runtime::date::new_date(env, value.into())
        };
        Handle::new_internal(JsDate(local))
    }

    /// Get the Date's value
    pub fn value<'a, C: Context<'a>>(self, cx: &mut C) -> f64 {
        let env = cx.env().to_raw();
        unsafe {
            neon_runtime::date::value(env, self.to_raw())
        }
    }

    /// Check if the Date's value is valid
    pub fn is_valid<'a, C: Context<'a>>(self, cx: &mut C) -> bool {
        let value = self.value(cx);
        value <= JsDate::MAX_VALUE && value >= JsDate::MIN_VALUE
    }
}

impl ValueInternal for JsDate {
    fn name() -> String { "object".to_string() }

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool {
        unsafe { neon_runtime::tag::is_date(env.to_raw(), other.to_raw()) }
    }
}

impl Object for JsDate { }
