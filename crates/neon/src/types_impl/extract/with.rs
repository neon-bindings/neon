use crate::{
    context::Cx,
    result::JsResult,
    types::{extract::TryIntoJs, Value},
};

/// Wraps a closure that will be lazily evaluated when [`TryIntoJs::try_into_js`] is
/// called.
///
/// Useful for executing arbitrary code on the main thread before returning from a
/// function exported with [`neon::export`](crate::export).
///
/// If you see the following error, due to [incorrect inference][issue], use [`with`]
/// instead.
///
/// [issue]: https://github.com/rust-lang/rust/issues/70263
///
/// ```text
/// error: implementation of `neon::types::extract::TryIntoJs` is not general enough
/// ```
///
/// ## Example
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
///     extract::with(move |cx| -> NeonResult<_> {
///         cx.global::<JsObject>("console")?
///             .method(cx, "log")?
///             .arg(&log)?
///             .exec()?;
///
///         sum.try_into_js(cx)
///     })
/// }
/// ```
pub struct With<F>(pub F);

/// Helper to ensure correct inference of [lifetime bounds][hrtb] on closures
/// provided to [`With<F>`](With) without needing [explicit annotation][binder].
///
/// **Note:** The return type is [`JsResult`]. If you need to return a non-JavaScript type,
/// call [`TryIntoJs::try_into_js`].
///
/// _See [`With`](With#Example) for example usage._
///
/// [hrtb]: https://doc.rust-lang.org/nomicon/hrtb.html
/// [binder]: https://rust-lang.github.io/rfcs/3216-closure-lifetime-binder.html
pub fn with<V, F>(f: F) -> With<F>
where
    V: Value,
    for<'cx> F: FnOnce(&mut Cx<'cx>) -> JsResult<'cx, V>,

    // N.B.: This bound ensures that the return type implements `TryIntoJs<'_>`
    // without making it an opaque `impl Trait`.
    for<'cx> With<F>: TryIntoJs<'cx, Value = V>,
{
    With(f)
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
