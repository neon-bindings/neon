use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    sync::Arc,
};

use crate::{
    context::FunctionContext,
    handle::{Handle, Root},
    object::Object,
    result::{NeonResult, Throw},
    types::{
        Value,
        extract::{Date, Error, TryIntoJs},
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

impl Sealed for &String {}

impl<'cx, V: Value> Sealed for Handle<'cx, V> {}

impl<O: Object> Sealed for Root<O> {}

impl<T> Sealed for Option<T> {}

impl<T, E> Sealed for Result<T, E> {}

impl<'cx, T> Sealed for Box<T> where T: TryIntoJs<'cx> {}

impl<T> Sealed for RefCell<T> {}

impl<T> Sealed for &RefCell<T> {}

impl<T> Sealed for Arc<T> {}

impl<T> Sealed for Rc<T> {}

impl<T> Sealed for Ref<'_, T> {}

impl<T> Sealed for RefMut<'_, T> {}

impl_sealed!(
    u8, u16, u32, i8, i16, i32, f32, f64, bool, String, Date, Throw, Error,
);
