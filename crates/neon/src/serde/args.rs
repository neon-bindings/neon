use serde::de;

use crate::{
    context::FunctionContext,
    handle::{Handle, Root},
    object::Object,
    result::NeonResult,
    types::{JsValue, Value},
};

pub trait FromArg<'cx>: Sized {
    fn from_arg(cx: &mut FunctionContext<'cx>, v: Handle<'cx, JsValue>) -> NeonResult<Self>;
}

impl<'cx, V> FromArg<'cx> for Handle<'cx, V>
where
    V: Value,
{
    fn from_arg(cx: &mut FunctionContext<'cx>, v: Handle<'cx, JsValue>) -> NeonResult<Self> {
        v.downcast_or_throw(cx)
    }
}

impl<'cx, V> FromArg<'cx> for V
where
    V: de::DeserializeOwned + ?Sized,
{
    fn from_arg(cx: &mut FunctionContext<'cx>, v: Handle<'cx, JsValue>) -> NeonResult<Self> {
        super::deserialize(cx, v)
    }
}

impl<'cx, O> FromArg<'cx> for Root<O>
where
    O: Object,
{
    fn from_arg(cx: &mut FunctionContext<'cx>, v: Handle<'cx, JsValue>) -> NeonResult<Self> {
        Ok(v.downcast_or_throw::<O, _>(cx)?.root(cx))
    }
}

pub trait FromArgs<'cx>: Sized {
    fn from_args(cx: &mut FunctionContext<'cx>) -> NeonResult<Self>;
}

impl<'cx> FromArgs<'cx> for () {
    fn from_args(_cx: &mut FunctionContext<'cx>) -> NeonResult<Self> {
        Ok(())
    }
}

macro_rules! impl_arguments {
    ([$($head:ident),*], []) => {};

    ([$($head:ident),*], [$cur:ident $(, $tail:ident)*]) => {
        impl<'cx, $($head,)* $cur> FromArgs<'cx> for ($($head,)* $cur,)
        where
            $($head: FromArg<'cx>,)*
            $cur: FromArg<'cx>,
        {
            fn from_args(cx: &mut FunctionContext<'cx>) -> NeonResult<Self> {
                #[allow(non_snake_case)]
                let [$($head,)* $cur] = cx.argv();

                Ok((
                    $($head::from_arg(cx, $head)?,)*
                    $cur::from_arg(cx, $cur)?,
                ))
            }
        }

        impl_arguments!([$($head,)* $cur], [$($tail),*]);
    };

    ($($name:ident),* $(,)*) => {
        impl_arguments!([], [$($name),*]);
    };
}

impl_arguments![
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20,
    T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31,
];
