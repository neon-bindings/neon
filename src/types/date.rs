use std;
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

/// A JavaScript Date object.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JsDate(raw::Local);

impl Value for JsDate { }

impl Managed for JsDate {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(_: Env, h: raw::Local) -> Self { JsDate(h) }
}

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct DateError(DateErrorKind);

impl DateError {
    pub fn kind(&self) -> DateErrorKind {
        self.0
    }
}

impl fmt::Display for DateError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0.as_str())
    }
}

impl Error for DateError {}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
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

impl<'a> JsResultExt<'a, JsDate> for Result<Handle<'a, JsDate>, DateError> {
    /// Creates a `Error` on error
    fn or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> JsResult<'a, JsDate> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => cx.throw_range_error(&e.to_string())
        }
    }
}

impl JsDate {
    pub const MIN_VALUE: f64 = -8.64e15;
    pub const MAX_VALUE: f64 = 8.64e15;

    pub fn new<'a, C: Context<'a>, T: Into<f64>>(cx: &mut C, value: T) -> Result<Handle<'a, JsDate>, DateError> {
        let env = cx.env().to_raw();
        let time = value.into();

        if time > JsDate::MAX_VALUE {
            return Err(DateError(DateErrorKind::Overflow))
        } else if time < JsDate::MIN_VALUE {
            return Err(DateError(DateErrorKind::Underflow))
        }

        let local = unsafe {
            neon_runtime::date::new_date(env, time.clone())
        };
        let date = Handle::new_internal(JsDate(local));
        Ok(date)
    }

    pub fn new_lossy<'a, C: Context<'a>, V: Into<f64> + std::cmp::PartialOrd>(cx: &mut C, value: V) -> Handle<'a, JsDate> {
        let env = cx.env().to_raw();
        let local = unsafe {
            neon_runtime::date::new_date(env, value.into())
        };
        Handle::new_internal(JsDate(local))
    }

    pub fn value<'a, C: Context<'a>>(self, cx: &mut C) -> f64 {
        let env = cx.env().to_raw();
        unsafe {
            neon_runtime::date::value(env, self.to_raw())
        }
    }

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
