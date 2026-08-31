use crate::{
    context::Cx,
    result::JsResult,
    types::{extract::TryIntoJs, Value},
};

struct With<F>(pub F);

/// Wraps a closure that will be lazily evaluated when [`TryIntoJs::try_into_js`] is
/// called.
///
/// Useful for executing arbitrary code on the main thread before returning from a
/// function exported with [`neon::export`](crate::export).
///
/// The closure must return [`JsResult`]. Prefer the
/// [`with!`](crate::types::extract::with!) macro, which accepts a body of any type that
/// implements [`TryIntoJs`] and converts it automatically.
pub fn with<V, F>(f: F) -> impl for<'cx> TryIntoJs<'cx, Value = V>
where
    V: Value,
    for<'cx> F: FnOnce(&mut Cx<'cx>) -> JsResult<'cx, V>,
{
    With(f)
}

/// Wraps a closure that will be lazily evaluated when
/// [`TryIntoJs::try_into_js`](crate::types::extract::TryIntoJs::try_into_js) is called.
///
/// Useful for executing arbitrary code on the main thread before returning from a
/// function exported with [`neon::export`](crate::export).
///
/// The value of the body is converted with
/// [`TryIntoJs`](crate::types::extract::TryIntoJs), following the same rules as the
/// return value of an exported function — e.g., `Err` becomes a JavaScript exception.
///
/// ## Example
///
/// ```
/// # use neon::{prelude::*, types::extract::{self, TryIntoJs}};
/// #[neon::export(task)]
/// fn sum(nums: Vec<f64>) -> impl for<'cx> TryIntoJs<'cx> {
///     let sum = nums.into_iter().sum::<f64>();
///
///     extract::with!(move |cx| cx.number(sum))
/// }
/// ```
///
/// Ordinary closure capture rules apply: `move` gives the closure ownership of the
/// variables it captures (`sum` above) and is required when the closure outlives the
/// enclosing function, as it does when returned from an exported function.
///
/// Fallible bodies may use `?`. Annotate the closure's return type to name the error
/// type:
///
/// ```
/// # use neon::{prelude::*, types::extract::{self, TryIntoJs}};
/// use std::time::Instant;
///
/// #[neon::export(task)]
/// fn sum(nums: Vec<f64>) -> impl for<'cx> TryIntoJs<'cx> {
///     let start = Instant::now();
///     let sum = nums.into_iter().sum::<f64>();
///     let log = format!("sum took {} ms", start.elapsed().as_millis());
///
///     extract::with!(move |cx| -> NeonResult<_> {
///         cx.global::<JsObject>("console")?
///             .method(cx, "log")?
///             .arg(&log)?
///             .exec()?;
///
///         Ok(sum)
///     })
/// }
/// ```
// `macro_rules!` macros cannot cross crate boundaries without `#[macro_export]`,
// which always exports at the crate root. Export under a hidden internal name;
// the `extract` module re-exports it as `with!`.
#[doc(hidden)]
#[macro_export]
macro_rules! __with {
    (move |$cx:ident| $body:expr) => {
        $crate::types::extract::with(move |$cx| {
            let __v = (|| $body)();
            $crate::types::extract::TryIntoJs::try_into_js(__v, $cx)
        })
    };
    (|$cx:ident| $body:expr) => {
        $crate::types::extract::with(|$cx| {
            let __v = (|| $body)();
            $crate::types::extract::TryIntoJs::try_into_js(__v, $cx)
        })
    };
    (move |_| $body:expr) => {
        $crate::types::extract::with(move |__cx| {
            let __v = (|| $body)();
            $crate::types::extract::TryIntoJs::try_into_js(__v, __cx)
        })
    };
    (|_| $body:expr) => {
        $crate::types::extract::with(|__cx| {
            let __v = (|| $body)();
            $crate::types::extract::TryIntoJs::try_into_js(__v, __cx)
        })
    };
    (move |$cx:ident| -> $ret:ty $body:block) => {
        $crate::types::extract::with(move |$cx| {
            let __v = (|| -> $ret { $body })();
            $crate::types::extract::TryIntoJs::try_into_js(__v, $cx)
        })
    };
    (|$cx:ident| -> $ret:ty $body:block) => {
        $crate::types::extract::with(|$cx| {
            let __v = (|| -> $ret { $body })();
            $crate::types::extract::TryIntoJs::try_into_js(__v, $cx)
        })
    };
    (move |_| -> $ret:ty $body:block) => {
        $crate::types::extract::with(move |__cx| {
            let __v = (|| -> $ret { $body })();
            $crate::types::extract::TryIntoJs::try_into_js(__v, __cx)
        })
    };
    (|_| -> $ret:ty $body:block) => {
        $crate::types::extract::with(|__cx| {
            let __v = (|| -> $ret { $body })();
            $crate::types::extract::TryIntoJs::try_into_js(__v, __cx)
        })
    };
}

impl<'cx, O, F> TryIntoJs<'cx> for With<F>
where
    O: TryIntoJs<'cx>,
    F: FnOnce(&mut Cx<'cx>) -> O,
{
    type Value = O::Value;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        (self.0)(cx).try_into_js(cx)
    }
}

impl<F> super::private::Sealed for With<F> {}
