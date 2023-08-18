//! Traits and utilities for extract Rust data from JavaScript values
//!
//! The full list of included extractors can be found on [`TryFromJs`].
//!
//! ## Extracting Handles
//!
//! JavaScript arguments may be extracted into a Rust tuple.
//!
//! ```
//! # use neon::prelude::*;
//! fn greet(mut cx: FunctionContext) -> JsResult<JsString> {
//!     let (greeting, name): (Handle<JsString>, Handle<JsString>) = cx.args()?;
//!     let message = format!("{}, {}!", greeting.value(&mut cx), name.value(&mut cx));
//!
//!     Ok(cx.string(message))
//! }
//! ```
//!
//! ## Extracting Native Types
//!
//! It's also possible to extract directly into native Rust types instead of a [`Handle`].
//!
//! ```
//! # use neon::prelude::*;
//! fn add(mut cx: FunctionContext) -> JsResult<JsNumber> {
//!     let (a, b): (f64, f64) = cx.args()?;
//!
//!     Ok(cx.number(a + b))
//! }
//! ```
//!
//! ## Extracting [`Option`]
//!
//! It's also possible to mix [`Handle`], Rust types, and even [`Option`] for
//! handling `null` and `undefined`.
//!
//! ```
//! # use neon::prelude::*;
//! fn get_or_default(mut cx: FunctionContext) -> JsResult<JsValue> {
//!     let (n, default_value): (Option<f64>, Handle<JsValue>) = cx.args()?;
//!
//!     if let Some(n) = n {
//!         return Ok(cx.number(n).upcast());
//!     }
//!
//!     Ok(default_value)
//! }
//! ```
//!
//! ## Additional Extractors
//!
//! In some cases, the expected JavaScript type is ambiguous. For example, when
//! trying to extract an [`f64`], the argument may be a `Date` instead of a `number`.
//! Newtype extractors are provided to help.
//!
//! ```
//! # use neon::prelude::*;
//! # #[cfg(feature = "napi-5")]
//! # use neon::types::JsDate;
//! # #[cfg(feature = "napi-5")]
//! use neon::extract::Date;
//!
//! # #[cfg(feature = "napi-5")]
//! fn add_hours(mut cx: FunctionContext) -> JsResult<JsDate> {
//!     const MS_PER_HOUR: f64 = 60.0 * 60.0 * 1000.0;
//!
//!     let (Date(date), hours): (Date, f64) = cx.args()?;
//!     let date = date + hours * MS_PER_HOUR;
//!
//!     cx.date(date).or_throw(&mut cx)
//! }
//! ```
//!
//! ## Overloaded Functions
//!
//! It's common in JavaScript to overload function signatures. This can be implemented with
//! custom extractors for each field, but it can also be done by attempting extraction
//! multiple times with [`cx.try_catch(..)`](Context::try_catch).
//!
//! ```
//! # use neon::prelude::*;
//!
//! fn add(mut cx: FunctionContext, a: f64, b: f64) -> Handle<JsNumber> {
//!     cx.number(a + b)
//! }
//!
//! fn concat(mut cx: FunctionContext, a: String, b: String) -> Handle<JsString> {
//!     cx.string(a + &b)
//! }
//!
//! fn combine(mut cx: FunctionContext) -> JsResult<JsValue> {
//!     if let Ok((a, b)) = cx.try_catch(|cx| cx.args()) {
//!         return Ok(add(cx, a, b).upcast());
//!     }
//!
//!     let (a, b) = cx.args()?;
//!
//!     Ok(concat(cx, a, b).upcast())
//! }
//! ```
//!
//! Note well, in this example, type annotations are not required on the tuple because
//! Rust is able to infer it from the type arguments on `add` and `concat`.

use std::ptr;

use crate::{
    context::{Context, FunctionContext},
    handle::{Handle, Root},
    object::Object,
    result::{NeonResult, ResultExt, Throw},
    sys,
    types::{
        buffer::{Binary, TypedArray},
        private::ValueInternal,
        Finalize, JsArrayBuffer, JsBox, JsTypedArray, JsValue, Value,
    },
};

#[cfg(feature = "napi-6")]
use crate::types::{bigint::Sign, JsBigInt};

/// Extract Rust data from a JavaScript value
pub trait TryFromJs<'cx>: Sized {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>;
}

impl<'cx, V> TryFromJs<'cx> for Handle<'cx, V>
where
    V: Value,
{
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        v.downcast_or_throw(cx)
    }
}

impl<'cx, T> TryFromJs<'cx> for Option<T>
where
    T: TryFromJs<'cx>,
{
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        if is_null_or_undefined(cx, v)? {
            return Ok(None);
        }

        T::try_from_js(cx, v).map(Some)
    }
}

impl<'cx> TryFromJs<'cx> for f64 {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        let mut n = 0f64;

        unsafe {
            match sys::get_value_double(cx.env().to_raw(), v.to_local(), &mut n) {
                sys::Status::NumberExpected => return cx.throw_type_error("number expected"),
                sys::Status::PendingException => return Err(Throw::new()),
                status => assert_eq!(status, sys::Status::Ok),
            }
        }

        Ok(n)
    }
}

impl<'cx> TryFromJs<'cx> for bool {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        let mut b = false;

        unsafe {
            match sys::get_value_bool(cx.env().to_raw(), v.to_local(), &mut b) {
                sys::Status::NumberExpected => return cx.throw_type_error("bool expected"),
                sys::Status::PendingException => return Err(Throw::new()),
                status => assert_eq!(status, sys::Status::Ok),
            }
        }

        Ok(b)
    }
}

impl<'cx> TryFromJs<'cx> for String {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        let env = cx.env().to_raw();
        let v = v.to_local();
        let mut len = 0usize;

        unsafe {
            match sys::get_value_string_utf8(env, v, ptr::null_mut(), 0, &mut len) {
                sys::Status::StringExpected => return cx.throw_type_error("string expected"),
                sys::Status::PendingException => return Err(Throw::new()),
                status => assert_eq!(status, sys::Status::Ok),
            }
        }

        // Make room for null terminator to avoid losing a character
        let mut buf = Vec::<u8>::with_capacity(len + 1);
        let mut written = 0usize;

        unsafe {
            assert_eq!(
                sys::get_value_string_utf8(
                    env,
                    v,
                    buf.as_mut_ptr().cast(),
                    buf.capacity(),
                    &mut written,
                ),
                sys::Status::Ok,
            );

            debug_assert_eq!(len, written);
            buf.set_len(len);

            Ok(String::from_utf8_unchecked(buf))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-5")))]
#[cfg(feature = "napi-5")]
/// Extract an [`f64`] from a [`Date`](crate::types::JsDate)
pub struct Date(pub f64);

#[cfg_attr(docsrs, doc(cfg(feature = "napi-5")))]
#[cfg(feature = "napi-5")]
impl<'cx> TryFromJs<'cx> for Date {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        let mut d = 0f64;

        unsafe {
            match sys::get_date_value(cx.env().to_raw(), v.to_local(), &mut d) {
                sys::Status::DateExpected => return cx.throw_type_error("Date expected"),
                sys::Status::PendingException => return Err(Throw::new()),
                status => assert_eq!(status, sys::Status::Ok),
            }
        }

        Ok(Date(d))
    }
}

impl<'cx, T> TryFromJs<'cx> for Vec<T>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        v.downcast_or_throw::<JsTypedArray<T>, _>(cx)
            .map(|v| v.as_slice(cx).to_vec())
    }
}

/// Extract a [`Vec<u8>`] from an [`ArrayBuffer`](JsArrayBuffer)
pub struct ArrayBuffer(pub Vec<u8>);

impl<'cx> TryFromJs<'cx> for ArrayBuffer {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        let buf = v
            .downcast_or_throw::<JsArrayBuffer, _>(cx)?
            .as_slice(cx)
            .to_vec();

        Ok(ArrayBuffer(buf))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
#[cfg(feature = "napi-6")]
impl<'cx> TryFromJs<'cx> for u64 {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        v.downcast_or_throw::<JsBigInt, _>(cx)?
            .to_u64(cx)
            .or_throw(cx)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
#[cfg(feature = "napi-6")]
impl<'cx> TryFromJs<'cx> for i64 {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        v.downcast_or_throw::<JsBigInt, _>(cx)?
            .to_i64(cx)
            .or_throw(cx)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
#[cfg(feature = "napi-6")]
impl<'cx> TryFromJs<'cx> for u128 {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        v.downcast_or_throw::<JsBigInt, _>(cx)?
            .to_u128(cx)
            .or_throw(cx)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
#[cfg(feature = "napi-6")]
impl<'cx> TryFromJs<'cx> for i128 {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        v.downcast_or_throw::<JsBigInt, _>(cx)?
            .to_i128(cx)
            .or_throw(cx)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
#[cfg(feature = "napi-6")]
/// Extract the [`Sign`] and [`u64`] words from a [`BigInt`](JsBigInt)
pub struct BigInt(pub Sign, pub Vec<u64>);

#[cfg(feature = "napi-6")]
impl<'cx> TryFromJs<'cx> for BigInt {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        let (sign, d) = v.downcast_or_throw::<JsBigInt, _>(cx)?.to_digits_le(cx);

        Ok(BigInt(sign, d))
    }
}

impl<'cx, T> TryFromJs<'cx> for &'cx T
where
    T: Finalize + 'static,
{
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        Ok(v.downcast_or_throw::<JsBox<T>, _>(cx)?.as_ref())
    }
}

impl<'cx> TryFromJs<'cx> for () {
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        if !is_null_or_undefined(cx, v)? {
            return cx.throw_type_error("expected null or undefined");
        }

        Ok(())
    }
}

impl<'cx, V> TryFromJs<'cx> for Root<V>
where
    V: Object,
{
    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        Ok(v.downcast_or_throw::<V, _>(cx)?.root(cx))
    }
}

fn is_null_or_undefined<'cx, C, V>(cx: &mut C, v: Handle<V>) -> NeonResult<bool>
where
    C: Context<'cx>,
    V: Value,
{
    let mut ty = sys::ValueType::Object;

    unsafe {
        match sys::typeof_value(cx.env().to_raw(), v.to_local(), &mut ty) {
            sys::Status::PendingException => return Err(Throw::new()),
            status => assert_eq!(status, sys::Status::Ok),
        }
    }

    Ok(matches!(
        ty,
        sys::ValueType::Undefined | sys::ValueType::Null,
    ))
}

/// Trait specifying values that may be extracted from function arguments.
///
/// **Note:** This trait is implemented for tuples of up to 32 values, but for
/// the sake of brevity, only tuples up to size 8 are shown in this documentation.
pub trait FromArgs<'cx>: private::FromArgsInternal<'cx> {}

macro_rules! impl_arguments {
    ($(#[$attrs:meta])? [$($head:ident),*], []) => {};

    ($(#[$attrs:meta])? [$($head:ident),*], [$cur:ident $(, $tail:ident)*]) => {
        $(#[$attrs])?
        impl<'cx, $($head,)* $cur> FromArgs<'cx> for ($($head,)* $cur,)
        where
            $($head: TryFromJs<'cx>,)*
            $cur: TryFromJs<'cx>,
        {}

        impl<'cx, $($head,)* $cur> private::FromArgsInternal<'cx> for ($($head,)* $cur,)
        where
            $($head: TryFromJs<'cx>,)*
            $cur: TryFromJs<'cx>,
        {
            fn from_args(cx: &mut FunctionContext<'cx>) -> NeonResult<Self> {
                #[allow(non_snake_case)]
                let [$($head,)* $cur] = cx.argv();

                Ok((
                    $($head::try_from_js(cx, $head)?,)*
                    $cur::try_from_js(cx, $cur)?,
                ))
            }
        }

        impl_arguments!($(#[$attrs])? [$($head,)* $cur], [$($tail),*]);
    };
}

impl_arguments!([], [T1, T2, T3, T4, T5, T6, T7, T8]);
impl_arguments!(
    #[doc(hidden)]
    [T1, T2, T3, T4, T5, T6, T7, T8],
    [
        T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26,
        T27, T28, T29, T30, T31, T32
    ]
);

mod private {
    use crate::{context::FunctionContext, result::NeonResult};

    pub trait FromArgsInternal<'cx>: Sized {
        fn from_args(cx: &mut FunctionContext<'cx>) -> NeonResult<Self>;
    }
}
