use crate::{
    context::Cx,
    handle::Handle,
    result::{JsResult, NeonResult},
    types::{
        buffer::{Binary, TypedArray},
        extract::{private, TryFromJs, TryIntoJs, TypeExpected},
        JsArrayBuffer, JsBigInt64Array, JsBigUint64Array, JsBuffer, JsFloat32Array, JsFloat64Array,
        JsInt16Array, JsInt32Array, JsInt8Array, JsTypedArray, JsUint16Array, JsUint32Array,
        JsUint8Array, JsValue, Value,
    },
};

/// Wrapper for converting between bytes and [`JsArrayBuffer`](JsArrayBuffer)
pub struct ArrayBuffer<B>(pub B);

impl<'cx, B> TryFromJs<'cx> for ArrayBuffer<B>
where
    for<'b> B: From<&'b [u8]>,
{
    type Error = TypeExpected<JsBuffer>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        let v = match v.downcast::<JsArrayBuffer, _>(cx) {
            Ok(v) => v,
            Err(_) => return Ok(Err(Self::Error::new())),
        };

        Ok(Ok(ArrayBuffer(B::from(v.as_slice(cx)))))
    }
}

impl<'cx, B> TryIntoJs<'cx> for ArrayBuffer<B>
where
    B: AsRef<[u8]>,
{
    type Value = JsArrayBuffer;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsArrayBuffer::from_slice(cx, self.0.as_ref())
    }
}

impl<B> private::Sealed for ArrayBuffer<B> {}

/// Wrapper for converting between bytes and [`JsBuffer`](JsBuffer)
pub struct Buffer<B>(pub B);

impl<'cx, B> TryFromJs<'cx> for Buffer<B>
where
    for<'b> B: From<&'b [u8]>,
{
    type Error = TypeExpected<JsBuffer>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        let v = match v.downcast::<JsBuffer, _>(cx) {
            Ok(v) => v,
            Err(_) => return Ok(Err(Self::Error::new())),
        };

        Ok(Ok(Buffer(B::from(v.as_slice(cx)))))
    }
}

impl<'cx, B> TryIntoJs<'cx> for Buffer<B>
where
    B: AsRef<[u8]>,
{
    type Value = JsBuffer;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsBuffer::from_slice(cx, self.0.as_ref())
    }
}

impl<B> private::Sealed for Buffer<B> {}

impl<'cx, T> TryIntoJs<'cx> for Vec<T>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Value = JsTypedArray<T>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsTypedArray::from_slice(cx, self.as_slice())
    }
}

impl<'cx, T> TryFromJs<'cx> for Vec<T>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Error = TypeExpected<JsTypedArray<T>>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        let v = match v.downcast::<JsTypedArray<T>, _>(cx) {
            Ok(v) => v,
            Err(_) => return Ok(Err(Self::Error::new())),
        };

        Ok(Ok(v.as_slice(cx).to_vec()))
    }
}

impl<T> private::Sealed for Vec<T>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
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

impl<T, const N: usize> private::Sealed for [T; N]
where
    JsTypedArray<T>: Value,
    T: Binary,
{
}

impl<'cx, T> TryIntoJs<'cx> for &[T]
where
    JsTypedArray<T>: Value,
    T: Binary,
{
    type Value = JsTypedArray<T>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        JsTypedArray::from_slice(cx, self)
    }
}

impl<T> private::Sealed for &[T]
where
    JsTypedArray<T>: Value,
    T: Binary,
{
}

macro_rules! typed_array {
    ($js:ident, $name:ident, $type:ty) => {
        #[doc = concat!(
            "Wrapper for converting between a Rust `[",
            stringify!($type),
            "]` array type and a [`",
            stringify!($js),
            "`]",
        )]
        pub struct $name<T>(pub T);

        impl<'cx, T> TryIntoJs<'cx> for $name<T>
        where
            T: AsRef<[$type]>,
        {
            type Value = $js;

            fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
                $js::from_slice(cx, self.0.as_ref())
            }
        }

        impl<'cx, T> TryFromJs<'cx> for $name<T>
        where
            for<'a> T: From<&'a [$type]>,
        {
            type Error = TypeExpected<$js>;

            fn try_from_js(
                cx: &mut Cx<'cx>,
                v: Handle<'cx, JsValue>,
            ) -> NeonResult<Result<Self, Self::Error>> {
                let v = match v.downcast::<$js, _>(cx) {
                    Ok(v) => v,
                    Err(_) => return Ok(Err(TypeExpected::new())),
                };

                Ok(Ok(Self(T::from(v.as_slice(cx)))))
            }
        }

        impl<T> private::Sealed for $name<T> {}
    };

    ($(($js:ident, $name:ident, $type:ty),)*) => {
        $(typed_array!($js, $name, $type);)*
    };
}

typed_array![
    (JsInt8Array, Int8Array, i8),
    (JsUint8Array, Uint8Array, u8),
    (JsInt16Array, Int16Array, i16),
    (JsUint16Array, Uint16Array, u16),
    (JsInt32Array, Int32Array, i32),
    (JsUint32Array, Uint32Array, u32),
    (JsFloat32Array, Float32Array, f32),
    (JsFloat64Array, Float64Array, f64),
    (JsBigInt64Array, BigInt64Array, i64),
    (JsBigUint64Array, BigUint64Array, u64),
];
