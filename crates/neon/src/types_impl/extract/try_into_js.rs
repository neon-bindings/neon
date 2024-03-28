use crate::{
    context::Context,
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

    fn try_into_js<C>(self, _cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        Ok(self)
    }
}

impl<'cx, T, E> TryIntoJs<'cx> for Result<T, E>
where
    T: TryIntoJs<'cx>,
    E: TryIntoJs<'cx>,
{
    type Value = T::Value;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
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

    fn try_into_js<C>(self, _cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        Err(self)
    }
}

impl<'cx, T> TryIntoJs<'cx> for Option<T>
where
    T: TryIntoJs<'cx>,
{
    type Value = JsValue;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        if let Some(val) = self {
            val.try_into_js(cx).map(|v| v.upcast())
        } else {
            Ok(cx.undefined().upcast())
        }
    }
}

macro_rules! impl_number {
    ($ty:ident) => {
        impl<'cx> TryIntoJs<'cx> for $ty {
            type Value = JsNumber;

            fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
            where
                C: Context<'cx>,
            {
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

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        Ok(cx.string(self))
    }
}

impl<'cx> TryIntoJs<'cx> for &'cx str {
    type Value = JsString;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        Ok(cx.string(self))
    }
}

impl<'cx, T> TryIntoJs<'cx> for Vec<T>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Value = JsTypedArray<T>;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        JsTypedArray::from_slice(cx, &self)
    }
}

impl<'cx, T> TryIntoJs<'cx> for &'cx [T]
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Value = JsTypedArray<T>;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        JsTypedArray::from_slice(cx, self)
    }
}

impl<'cx> TryIntoJs<'cx> for bool {
    type Value = JsBoolean;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        Ok(cx.boolean(self))
    }
}

impl<'cx> TryIntoJs<'cx> for () {
    type Value = JsUndefined;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        Ok(cx.undefined())
    }
}

impl<'cx> TryIntoJs<'cx> for ArrayBuffer {
    type Value = JsArrayBuffer;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        JsArrayBuffer::from_slice(cx, &self.0)
    }
}

impl<'cx> TryIntoJs<'cx> for Buffer {
    type Value = JsBuffer;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        JsBuffer::from_slice(cx, &self.0)
    }
}

impl<'cx> TryIntoJs<'cx> for Date {
    type Value = JsDate;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Value>
    where
        C: Context<'cx>,
    {
        cx.date(self.0).or_throw(cx)
    }
}
