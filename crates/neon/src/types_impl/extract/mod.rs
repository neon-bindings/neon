//! Traits and utilities for extract Rust data from JavaScript values
//!
//! The full list of included extractors can be found on [`TryFromJs`].
//!
//! ## Extracting Handles
//!
//! JavaScript arguments may be extracted into a Rust tuple.
//!
//! ```
//! # use neon::{prelude::*, types::extract::*};
//! fn greet(mut cx: FunctionContext) -> JsResult<JsString> {
//!     let (Val::<JsString>(greeting), Val::<JsString>(name)) = cx.args()?;
//!     let message = format!("{}, {}!", greeting.value(&mut cx), name.value(&mut cx));
//!
//!     Ok(cx.string(message))
//! }
//! ```
//!
//! ## Extracting Native Types
//!
//! It's also possible to extract directly into native Rust types instead of a [`Handle`].
//!
//! ```
//! # use neon::{prelude::*, types::extract::*};
//! fn add(mut cx: FunctionContext) -> JsResult<JsNumber> {
//!     let (number(a), number(b)) = cx.args()?;
//!
//!     Ok(cx.number(a + b))
//! }
//! ```
//!
//! ## Extracting [`Option`]
//!
//! It's also possible to mix [`Handle`], Rust types, and even [`Option`] for
//! handling `null` and `undefined`.
//!
//! ```
//! # use neon::{prelude::*, types::extract::*};
//! fn get_or_default(mut cx: FunctionContext) -> JsResult<JsValue> {
//!     let (Opt::<f64>(n), Val::<JsValue>(default_value)) = cx.args()?;
//!
//!     if let Some(n) = n {
//!         return Ok(cx.number(n).upcast());
//!     }
//!
//!     Ok(default_value)
//! }
//! ```
//!
//! ## Additional Extractors
//!
//! In some cases, the expected JavaScript type is ambiguous. For example, when
//! trying to extract an [`f64`], the argument may be a `Date` instead of a `number`.
//! Newtype extractors are provided to help.
//!
//! ```
//! # use neon::{prelude::*, types::extract::*};
//! # #[cfg(feature = "napi-5")]
//! # use neon::types::JsDate;
//!
//! # #[cfg(feature = "napi-5")]
//! fn add_hours(mut cx: FunctionContext) -> JsResult<JsDate> {
//!     const MS_PER_HOUR: f64 = 60.0 * 60.0 * 1000.0;
//!
//!     let (Date(date), number(hours)) = cx.args()?;
//!     let date = date + hours * MS_PER_HOUR;
//!
//!     cx.date(date).or_throw(&mut cx)
//! }
//! ```
//!
//! ## Overloaded Functions
//!
//! It's common in JavaScript to overload function signatures. This can be implemented with
//! [`FunctionContext::args_opt`] or [`Context::try_catch`].
//!
//! ```
//! # use neon::{prelude::*, types::extract::*};
//!
//! fn add(mut cx: FunctionContext, a: f64, b: f64) -> Handle<JsNumber> {
//!     cx.number(a + b)
//! }
//!
//! fn concat(mut cx: FunctionContext, a: String, b: String) -> Handle<JsString> {
//!     cx.string(a + &b)
//! }
//!
//! fn combine(mut cx: FunctionContext) -> JsResult<JsValue> {
//!     if let Some((a, b)) = cx.args_opt()? {
//!         return Ok(add(cx, a, b).upcast());
//!     }
//!
//!     let (a, b) = cx.args()?;
//!
//!     Ok(concat(cx, a, b).upcast())
//! }
//! ```
//!
//! Note well, in this example, type annotations are not required on the tuple because
//! Rust is able to infer it from the type arguments on `add` and `concat`.

use crate::{
    context::{Context, FunctionContext},
    handle::Handle,
    result::NeonResult,
    types::JsValue,
};

pub use self::types::*;

mod types;

/// Extract Rust data from a JavaScript value
pub trait TryFromJs<'cx>: private::Sealed + Sized {
    type Error;

    fn try_from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Result<Self, Self::Error>>
    where
        C: Context<'cx>;

    fn from_js<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>;
}

/// Trait specifying values that may be extracted from function arguments.
///
/// **Note:** This trait is implemented for tuples of up to 32 values, but for
/// the sake of brevity, only tuples up to size 8 are shown in this documentation.
pub trait FromArgs<'cx>: private::FromArgsInternal<'cx> {}

macro_rules! impl_arguments {
    ($(#[$attrs:meta])? [$($head:ident),*], []) => {};

    ($(#[$attrs:meta])? [$($head:ident),*], [$cur:ident $(, $tail:ident)*]) => {
        $(#[$attrs])?
        impl<'cx, $($head,)* $cur> FromArgs<'cx> for ($($head,)* $cur,)
        where
            $($head: TryFromJs<'cx>,)*
            $cur: TryFromJs<'cx>,
        {}

        impl<'cx, $($head,)* $cur> private::FromArgsInternal<'cx> for ($($head,)* $cur,)
        where
            $($head: TryFromJs<'cx>,)*
            $cur: TryFromJs<'cx>,
        {
            fn from_args(cx: &mut FunctionContext<'cx>) -> NeonResult<Self> {
                #[allow(non_snake_case)]
                let [$($head,)* $cur] = cx.argv();

                Ok((
                    $($head::from_js(cx, $head)?,)*
                    $cur::from_js(cx, $cur)?,
                ))
            }

            fn from_args_opt(cx: &mut FunctionContext<'cx>) -> NeonResult<Option<Self>> {
                #[allow(non_snake_case)]
                let [$($head,)* $cur] = cx.argv();

                Ok(Some((
                    $(
                        match $head::try_from_js(cx, $head)? {
                            Ok(v) => v,
                            Err(_) => return Ok(None),
                        },
                    )*
                    match $cur::try_from_js(cx, $cur)? {
                        Ok(v) => v,
                        Err(_) => return Ok(None),
                    },
                )))
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
    use crate::{context::FunctionContext, result::NeonResult};

    pub trait Sealed {}

    pub trait FromArgsInternal<'cx>: Sized {
        fn from_args(cx: &mut FunctionContext<'cx>) -> NeonResult<Self>;

        fn from_args_opt(cx: &mut FunctionContext<'cx>) -> NeonResult<Option<Self>>;
    }
}
