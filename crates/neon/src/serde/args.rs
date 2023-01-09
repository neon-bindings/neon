use serde::de;

use crate::{
    context::FunctionContext,
    handle::{Handle, Root},
    object::Object,
    result::NeonResult,
    types::{JsValue, Value},
};

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
/// Trait specifying that a value may be deserialized from a function argument.
pub trait FromArg<'cx>: private::FromArgInternal<'cx> {}

impl<'cx, V> FromArg<'cx> for Handle<'cx, V> where V: Value {}

impl<'cx, V> private::FromArgInternal<'cx> for Handle<'cx, V>
where
    V: Value,
{
    fn from_arg(cx: &mut FunctionContext<'cx>, v: Handle<'cx, JsValue>) -> NeonResult<Self> {
        v.downcast_or_throw(cx)
    }
}

impl<'cx, V> FromArg<'cx> for V where V: de::DeserializeOwned + ?Sized {}

impl<'cx, V> private::FromArgInternal<'cx> for V
where
    V: de::DeserializeOwned + ?Sized,
{
    fn from_arg(cx: &mut FunctionContext<'cx>, v: Handle<'cx, JsValue>) -> NeonResult<Self> {
        super::deserialize(cx, v)
    }
}

impl<'cx, O> FromArg<'cx> for Root<O> where O: Object {}

impl<'cx, O> private::FromArgInternal<'cx> for Root<O>
where
    O: Object,
{
    fn from_arg(cx: &mut FunctionContext<'cx>, v: Handle<'cx, JsValue>) -> NeonResult<Self> {
        Ok(v.downcast_or_throw::<O, _>(cx)?.root(cx))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
/// Trait specifying values that may be deserialized from function arguments.
///
/// **Note:** This trait is implemented for tuples of up to 32 values, but for
/// the sake of brevity, only tuples up to size 8 are shown in this documentation.
pub trait FromArgs<'cx>: private::FromArgsInternal<'cx> {}

impl<'cx> FromArgs<'cx> for () {}

impl<'cx> private::FromArgsInternal<'cx> for () {
    fn from_args(_cx: &mut FunctionContext<'cx>) -> NeonResult<Self> {
        Ok(())
    }
}

pub(crate) fn from_args<'cx, T>(cx: &mut FunctionContext<'cx>) -> NeonResult<T>
where
    T: FromArgs<'cx>,
{
    private::FromArgsInternal::from_args(cx)
}

macro_rules! impl_arguments {
    ($(#[$attrs:meta])? [$($head:ident),*], []) => {};

    ($(#[$attrs:meta])? [$($head:ident),*], [$cur:ident $(, $tail:ident)*]) => {
        $(#[$attrs])?
        impl<'cx, $($head,)* $cur> FromArgs<'cx> for ($($head,)* $cur,)
        where
            $($head: FromArg<'cx>,)*
            $cur: FromArg<'cx>,
        {}

        impl<'cx, $($head,)* $cur> private::FromArgsInternal<'cx> for ($($head,)* $cur,)
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
    use crate::{context::FunctionContext, handle::Handle, result::NeonResult, types::JsValue};

    pub trait FromArgInternal<'cx>: Sized {
        fn from_arg(cx: &mut FunctionContext<'cx>, v: Handle<'cx, JsValue>) -> NeonResult<Self>;
    }

    pub trait FromArgsInternal<'cx>: Sized {
        fn from_args(cx: &mut FunctionContext<'cx>) -> NeonResult<Self>;
    }
}
