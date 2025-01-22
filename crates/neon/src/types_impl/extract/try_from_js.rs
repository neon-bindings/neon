// Implementations in this file are equivalent to a call to `.downcast()` and
// `.value(&mut cx)`. These specialized versions provide a performance benefit
// because they can combine two Node-API calls into a single call that both
// gets the value and checks the type at the same time.

use std::{convert::Infallible, ptr};

use crate::{
    context::{internal::ContextInternal, Cx},
    handle::{Handle, Root},
    object::Object,
    result::{NeonResult, Throw},
    sys,
    types::{
        extract::{Date, TryFromJs, TypeExpected},
        private::ValueInternal,
        JsBoolean, JsNumber, JsString, JsValue, Value,
    },
};

#[cfg(feature = "napi-5")]
use crate::types::JsDate;

impl<'cx, V> TryFromJs<'cx> for Handle<'cx, V>
where
    V: Value,
{
    type Error = TypeExpected<V>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        Ok(v.downcast(cx).map_err(|_| TypeExpected::new()))
    }
}

impl<'cx, O> TryFromJs<'cx> for Root<O>
where
    O: Object,
{
    type Error = TypeExpected<O>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        Ok(match v.downcast::<O, _>(cx) {
            Ok(v) => Ok(v.root(cx)),
            Err(_) => Err(TypeExpected::new()),
        })
    }
}

impl<'cx, T> TryFromJs<'cx> for Option<T>
where
    T: TryFromJs<'cx>,
{
    type Error = T::Error;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        if is_null_or_undefined(cx, v)? {
            return Ok(Ok(None));
        }

        T::try_from_js(cx, v).map(|v| v.map(Some))
    }
}

impl<'cx> TryFromJs<'cx> for f64 {
    type Error = TypeExpected<JsNumber>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        let mut n = 0f64;

        unsafe {
            match sys::get_value_double(cx.env().to_raw(), v.to_local(), &mut n) {
                Err(sys::Status::NumberExpected) => return Ok(Err(TypeExpected::new())),
                Err(sys::Status::PendingException) => return Err(Throw::new()),
                status => status.unwrap(),
            };
        }

        Ok(Ok(n))
    }
}

impl<'cx> TryFromJs<'cx> for bool {
    type Error = TypeExpected<JsBoolean>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        let mut b = false;

        unsafe {
            match sys::get_value_bool(cx.env().to_raw(), v.to_local(), &mut b) {
                Err(sys::Status::BooleanExpected) => return Ok(Err(TypeExpected::new())),
                Err(sys::Status::PendingException) => return Err(Throw::new()),
                status => status.unwrap(),
            };
        }

        Ok(Ok(b))
    }
}

impl<'cx> TryFromJs<'cx> for String {
    type Error = TypeExpected<JsString>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        let env = cx.env().to_raw();
        let v = v.to_local();
        let mut len = 0usize;

        unsafe {
            match sys::get_value_string_utf8(env, v, ptr::null_mut(), 0, &mut len) {
                Err(sys::Status::StringExpected) => return Ok(Err(TypeExpected::new())),
                Err(sys::Status::PendingException) => return Err(Throw::new()),
                status => status.unwrap(),
            };
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
                Ok(())
            );

            debug_assert_eq!(len, written);
            buf.set_len(len);

            Ok(Ok(String::from_utf8_unchecked(buf)))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-5")))]
#[cfg(feature = "napi-5")]
impl<'cx> TryFromJs<'cx> for Date {
    type Error = TypeExpected<JsDate>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        let mut d = 0f64;

        unsafe {
            match sys::get_date_value(cx.env().to_raw(), v.to_local(), &mut d) {
                Err(sys::Status::DateExpected) => return Ok(Err(TypeExpected::new())),
                Err(sys::Status::PendingException) => return Err(Throw::new()),
                status => status.unwrap(),
            };
        }

        Ok(Ok(Date(d)))
    }
}

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

    fn try_from_js(
        _cx: &mut Cx<'cx>,
        _v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        Ok(Ok(()))
    }
}

fn is_null_or_undefined<V>(cx: &mut Cx, v: Handle<V>) -> NeonResult<bool>
where
    V: Value,
{
    let mut ty = sys::ValueType::Object;

    unsafe {
        match sys::typeof_value(cx.env().to_raw(), v.to_local(), &mut ty) {
            Err(sys::Status::PendingException) => return Err(Throw::new()),
            status => status.unwrap(),
        };
    }

    Ok(matches!(
        ty,
        sys::ValueType::Undefined | sys::ValueType::Null,
    ))
}
