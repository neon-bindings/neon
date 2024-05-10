use crate::{
    context::FunctionContext,
    handle::Handle,
    result::{NeonResult, Throw},
    types::{
        buffer::Binary,
        extract::{ArrayBuffer, Buffer, Date, Error},
        JsTypedArray, Value,
    },
};

pub trait Sealed {}

pub trait FromArgsInternal<'cx>: Sized {
    fn from_args(cx: &mut FunctionContext<'cx>) -> NeonResult<Self>;

    fn from_args_opt(cx: &mut FunctionContext<'cx>) -> NeonResult<Option<Self>>;
}

macro_rules! impl_sealed {
    ($ty:ident) => {
        impl Sealed for $ty {}
    };

    ($($ty:ident),* $(,)*) => {
        $(
            impl_sealed!($ty);
        )*
    }
}

impl Sealed for () {}

impl Sealed for &str {}

impl<'cx, V: Value> Sealed for Handle<'cx, V> {}

impl<T> Sealed for Option<T> {}

impl<T, E> Sealed for Result<T, E> {}

impl<T> Sealed for Vec<T>
where
    JsTypedArray<T>: Value,
    T: Binary,
{
}

impl<T> Sealed for &[T]
where
    JsTypedArray<T>: Value,
    T: Binary,
{
}

impl_sealed!(
    u8,
    u16,
    u32,
    i8,
    i16,
    i32,
    f32,
    f64,
    bool,
    String,
    Date,
    Buffer,
    ArrayBuffer,
    Throw,
    Error,
);
