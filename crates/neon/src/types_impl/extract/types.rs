// Implementations in this file are equivalent to a call to `.downcast()` and
// `.value(&mut cx)`. These specialized versions provide a performance benefit
// because they can combine two Node-API calls into a single call that both
// gets the value and checks the type at the same time.

use std::{convert::Infallible, error, fmt, marker::PhantomData, ptr};

use crate::{
    context::Context,
    handle::Handle,
    result::{NeonResult, ResultExt, Throw},
    sys,
    types::{
        buffer::{Binary, TypedArray},
        extract::{private, TryFromJs},
        private::ValueInternal,
        JsArrayBuffer, JsBoolean, JsBuffer, JsNumber, JsString, JsTypedArray, JsValue, Value,
    },
};

#[cfg(feature = "napi-5")]
use crate::types::JsDate;

/// Error returned when a JavaScript value is not the type expected
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

// This implementation primarily exists for macro authors. It is infallible, rather
// than checking a type, to match the JavaScript conventions of ignoring additional
// arguments.
//
// N.B.: There is a blanket impl of `FromArgs` for `T` where `T: TryFromJs` to make
// the common case of `arity == 1` more ergonomic and avoid `(T)` is *not* a tuple
// foot-gun (but, `(T,)` is). This creates ambiguity for `()`. Are we extracting
// unit from the first argument of a function with `arity == 1` or is this a function
// with `arity == 0`? By making extraction of unit infallible, we eliminate any
// impact from the ambiguity.
impl<'cx> TryFromJs<'cx> for () {
    type Error = Infallible;

    fn try_from_js<C>(
        _cx: &mut C,
        _v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        Ok(Ok(()))
    }

    fn from_js<C>(_cx: &mut C, _v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        Ok(())
    }
}

impl private::Sealed for () {}

impl<'cx, T> TryFromJs<'cx> for Vec<T>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Error = TypeExpected<JsTypedArray<T>>;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        let v = match v.downcast::<JsTypedArray<T>, _>(cx) {
            Ok(v) => v,
            Err(_) => return Ok(Err(Self::Error::new())),
        };

        Ok(Ok(v.as_slice(cx).to_vec()))
    }

    from_js!();
}

impl<T> private::Sealed for Vec<T>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
}

/// Extract a [`Vec<u8>`] from a [`JsBuffer`]
pub struct Buffer(pub Vec<u8>);

impl<'cx> TryFromJs<'cx> for Buffer {
    type Error = TypeExpected<JsBuffer>;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        let v = match v.downcast::<JsBuffer, _>(cx) {
            Ok(v) => v,
            Err(_) => return Ok(Err(Self::Error::new())),
        };

        Ok(Ok(Buffer(v.as_slice(cx).to_vec())))
    }

    from_js!();
}

impl private::Sealed for Buffer {}

/// Extract a [`Vec<u8>`] from a [`JsArrayBuffer`]
pub struct ArrayBuffer(pub Vec<u8>);

impl<'cx> TryFromJs<'cx> for ArrayBuffer {
    type Error = TypeExpected<JsBuffer>;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>,
    {
        let v = match v.downcast::<JsArrayBuffer, _>(cx) {
            Ok(v) => v,
            Err(_) => return Ok(Err(Self::Error::new())),
        };

        Ok(Ok(ArrayBuffer(v.as_slice(cx).to_vec())))
    }

    from_js!();
}

impl private::Sealed for ArrayBuffer {}

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
