use std::{error, fmt, marker::PhantomData, ptr};

use crate::{
    context::Context,
    handle::Handle,
    result::{NeonResult, ResultExt, Throw},
    sys,
    types::{
        extract::{private, TryFromJs},
        private::ValueInternal,
        JsBoolean, JsNumber, JsString, JsValue, Value,
    },
};

#[cfg(feature = "napi-5")]
use crate::types::JsDate;

pub struct TypeExpected<T: Value>(PhantomData<T>);

impl<T: Value> TypeExpected<T> {
    fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Value> fmt::Display for TypeExpected<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "expected {}", T::name())
    }
}

impl<T: Value> fmt::Debug for TypeExpected<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("TypeExpected").field(&T::name()).finish()
    }
}

impl<T: Value> error::Error for TypeExpected<T> {}

impl<T, U: Value> ResultExt<T> for Result<T, TypeExpected<U>> {
    fn or_throw<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => cx.throw_type_error(format!("expected {}", U::name())),
        }
    }
}

macro_rules! delegate {
    ($target:ident, $source:ident) => {
        impl<'cx> TryFromJs<'cx> for $target {
            type Error = <$source as TryFromJs<'cx>>::Error;

            fn try_from_js<C>(
                cx: &mut C,
                v: Handle<'cx, JsValue>,
            ) -> NeonResult<Result<Self, Self::Error>>
            where
                C: Context<'cx>,
            {
                $source::try_from_js(cx, v).map(|v| v.map($target))
            }

            from_js!();
        }

        impl private::Sealed for $target {}
    };
}

macro_rules! from_js {
    () => {
        fn from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
        where
            C: Context<'cx>,
        {
            Self::try_from_js(cx, v)?.or_throw(cx)
        }
    };
}

/// Wrapper for extracting a JavaScript [`Handle`]
pub struct Val<'cx, V: Value>(pub Handle<'cx, V>)
where
    V: Value;

impl<'cx, V> TryFromJs<'cx> for Val<'cx, V>
where
    V: Value,
{
    type Error = TypeExpected<V>;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        Ok(v.downcast(cx).map(Self).map_err(|_| TypeExpected::new()))
    }

    from_js!();
}

impl<'cx, V: Value> private::Sealed for Val<'cx, V> {}

impl<'cx, V> TryFromJs<'cx> for Handle<'cx, V>
where
    V: Value,
{
    type Error = TypeExpected<V>;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        Ok(v.downcast(cx).map_err(|_| TypeExpected::new()))
    }

    from_js!();
}

impl<'cx, V: Value> private::Sealed for Handle<'cx, V> {}

/// Wrapper for extracting an [`Option`] from a value
pub struct Opt<T>(pub Option<T>)
where
    for<'cx> T: TryFromJs<'cx>;

impl<'cx, T> TryFromJs<'cx> for Opt<T>
where
    for<'a> T: TryFromJs<'a>,
{
    type Error = <Option<T> as TryFromJs<'cx>>::Error;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        <Option<T> as TryFromJs<'cx>>::try_from_js(cx, v).map(|v| v.map(Self))
    }

    fn from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        <Option<T> as TryFromJs<'cx>>::from_js(cx, v).map(Self)
    }
}

impl<T> private::Sealed for Opt<T> where for<'cx> T: TryFromJs<'cx> {}

impl<'cx, T> TryFromJs<'cx> for Option<T>
where
    T: TryFromJs<'cx>,
{
    type Error = T::Error;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        if is_null_or_undefined(cx, v)? {
            return Ok(Ok(None));
        }

        T::try_from_js(cx, v).map(|v| v.map(Some))
    }

    fn from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        if is_null_or_undefined(cx, v)? {
            return Ok(None);
        }

        T::from_js(cx, v).map(Some)
    }
}

impl<'cx, T> private::Sealed for Option<T> where T: TryFromJs<'cx> {}

#[allow(non_camel_case_types)]
/// Extract an [`f64`] from a [`JsNumber`]
pub struct number(pub f64);

delegate!(number, f64);

impl<'cx> TryFromJs<'cx> for f64 {
    type Error = TypeExpected<JsNumber>;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        let mut n = 0f64;

        unsafe {
            match sys::get_value_double(cx.env().to_raw(), v.to_local(), &mut n) {
                sys::Status::NumberExpected => return Ok(Err(TypeExpected::new())),
                sys::Status::PendingException => return Err(Throw::new()),
                status => assert_eq!(status, sys::Status::Ok),
            }
        }

        Ok(Ok(n))
    }

    from_js!();
}

impl private::Sealed for f64 {}

#[allow(non_camel_case_types)]
/// Extract a [`bool`] from a [`JsBoolean`]
pub struct boolean(pub bool);

delegate!(boolean, bool);

impl<'cx> TryFromJs<'cx> for bool {
    type Error = TypeExpected<JsBoolean>;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        let mut b = false;

        unsafe {
            match sys::get_value_bool(cx.env().to_raw(), v.to_local(), &mut b) {
                sys::Status::BooleanExpected => return Ok(Err(TypeExpected::new())),
                sys::Status::PendingException => return Err(Throw::new()),
                status => assert_eq!(status, sys::Status::Ok),
            }
        }

        Ok(Ok(b))
    }

    from_js!();
}

impl private::Sealed for bool {}

#[allow(non_camel_case_types)]
/// Extract a [`String`] from a [`JsString`]
pub struct string(pub String);

delegate!(string, String);

impl<'cx> TryFromJs<'cx> for String {
    type Error = TypeExpected<JsString>;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        let env = cx.env().to_raw();
        let v = v.to_local();
        let mut len = 0usize;

        unsafe {
            match sys::get_value_string_utf8(env, v, ptr::null_mut(), 0, &mut len) {
                sys::Status::StringExpected => return Ok(Err(TypeExpected::new())),
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

            Ok(Ok(String::from_utf8_unchecked(buf)))
        }
    }

    from_js!();
}

impl private::Sealed for String {}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-5")))]
#[cfg(feature = "napi-5")]
/// Extract an [`f64`] from a [`JsDate`]
pub struct Date(pub f64);

#[cfg_attr(docsrs, doc(cfg(feature = "napi-5")))]
#[cfg(feature = "napi-5")]
impl<'cx> TryFromJs<'cx> for Date {
    type Error = TypeExpected<JsDate>;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        let mut d = 0f64;

        unsafe {
            match sys::get_date_value(cx.env().to_raw(), v.to_local(), &mut d) {
                sys::Status::DateExpected => return Ok(Err(TypeExpected::new())),
                sys::Status::PendingException => return Err(Throw::new()),
                status => assert_eq!(status, sys::Status::Ok),
            }
        }

        Ok(Ok(Date(d)))
    }

    from_js!();
}

impl private::Sealed for Date {}

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
