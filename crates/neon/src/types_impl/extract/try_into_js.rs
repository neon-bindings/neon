use std::sync::Arc;

use crate::{
    context::{Context, Cx},
    handle::Handle,
    result::{JsResult, ResultExt, Throw},
    types::{
        buffer::Binary,
        extract::{ArrayBuffer, Buffer, Date, TryIntoJs},
        JsArrayBuffer, JsBoolean, JsBuffer, JsDate, JsNumber, JsString, JsTypedArray, JsUndefined,
        JsValue, Value,
    },
};

impl<'cx, T> TryIntoJs<'cx> for Handle<'cx, T>
where
    T: Value,
{
    type Value = T;

    fn try_into_js(self, _cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(self)
    }
}

impl<'cx, T, E> TryIntoJs<'cx> for Result<T, E>
where
    T: TryIntoJs<'cx>,
    E: TryIntoJs<'cx>,
{
    type Value = T::Value;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        match self {
            Ok(v) => v.try_into_js(cx),
            Err(err) => {
                let err = err.try_into_js(cx)?;

                cx.throw(err)
            }
        }
    }
}

impl<'cx> TryIntoJs<'cx> for Throw {
    type Value = JsValue;

    fn try_into_js(self, _cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Err(self)
    }
}

impl<'cx, T> TryIntoJs<'cx> for Option<T>
where
    T: TryIntoJs<'cx>,
{
    type Value = JsValue;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        if let Some(val) = self {
            val.try_into_js(cx).map(|v| v.upcast())
        } else {
            Ok(cx.undefined().upcast())
        }
    }
}

impl<'cx, T> TryIntoJs<'cx> for Box<T>
where
    T: TryIntoJs<'cx>,
{
    type Value = T::Value;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        (*self).try_into_js(cx)
    }
}

impl<'cx, T, V> TryIntoJs<'cx> for Arc<T>
where
    for<'a> &'a T: TryIntoJs<'cx, Value = V>,
    V: Value,
{
    type Value = V;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        self.as_ref().try_into_js(cx)
    }
}

macro_rules! impl_number {
    ($ty:ident) => {
        impl<'cx> TryIntoJs<'cx> for $ty {
            type Value = JsNumber;

            fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
                Ok(cx.number(self))
            }
        }
    };

    ($($ty:ident),* $(,)?) => {
        $(
            impl_number!($ty);
        )*
    }
}

impl_number!(u8, u16, u32, i8, i16, i32, f32, f64);

impl<'cx> TryIntoJs<'cx> for String {
    type Value = JsString;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(cx.string(self))
    }
}

impl<'a, 'cx> TryIntoJs<'cx> for &'a str {
    type Value = JsString;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(cx.string(self))
    }
}

impl<'a, 'cx> TryIntoJs<'cx> for &'a String {
    type Value = JsString;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(cx.string(self))
    }
}

impl<'cx, T> TryIntoJs<'cx> for Vec<T>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Value = JsTypedArray<T>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsTypedArray::from_slice(cx, &self)
    }
}

impl<'cx, T> TryIntoJs<'cx> for Box<[T]>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Value = JsTypedArray<T>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsTypedArray::from_slice(cx, &self)
    }
}

impl<'cx, T, const N: usize> TryIntoJs<'cx> for [T; N]
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Value = JsTypedArray<T>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsTypedArray::from_slice(cx, self.as_slice())
    }
}

impl<'a, 'cx, T> TryIntoJs<'cx> for &'a Vec<T>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Value = JsTypedArray<T>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsTypedArray::from_slice(cx, self)
    }
}

impl<'a, 'cx, T> TryIntoJs<'cx> for &'a [T]
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Value = JsTypedArray<T>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsTypedArray::from_slice(cx, self)
    }
}

impl<'cx> TryIntoJs<'cx> for bool {
    type Value = JsBoolean;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(cx.boolean(self))
    }
}

impl<'cx> TryIntoJs<'cx> for () {
    type Value = JsUndefined;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(cx.undefined())
    }
}

impl<'cx> TryIntoJs<'cx> for ArrayBuffer {
    type Value = JsArrayBuffer;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsArrayBuffer::from_slice(cx, &self.0)
    }
}

impl<'cx> TryIntoJs<'cx> for Buffer {
    type Value = JsBuffer;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsBuffer::from_slice(cx, &self.0)
    }
}

impl<'cx> TryIntoJs<'cx> for Date {
    type Value = JsDate;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        cx.date(self.0).or_throw(cx)
    }
}
