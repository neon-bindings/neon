use crate::{context::Cx, result::JsResult, types::extract::TryIntoJs};

/// Wraps a closure that will be lazily evaluated when [`TryIntoJs::try_into_js`] is
/// called.
///
/// Useful for executing arbitrary code on the main thread before returning from a
/// function exported with [`neon::export`](crate::export).
///
/// ## Example
///
/// ```
/// # use neon::{prelude::*, types::extract::{TryIntoJs, With}};
/// use std::time::Instant;
///
/// #[neon::export(task)]
/// fn sum(nums: Vec<f64>) -> impl for<'cx> TryIntoJs<'cx> {
///     let start = Instant::now();
///     let sum = nums.into_iter().sum::<f64>();
///     let log = format!("sum took {} ms", start.elapsed().as_millis());
///
///     With(move |cx| -> NeonResult<_> {
///         cx.global::<JsObject>("console")?
///             .method(cx, "log")?
///             .arg(&log)?
///             .exec()?;
///
///         Ok(sum)
///     })
/// }
/// ```
pub struct With<F, O>(pub F)
where
    // N.B.: We include additional required bounds to allow the compiler to infer the
    // correct closure argument when using `impl for<'cx> TryIntoJs<'cx>`. Without
    // these bounds, it would be necessary to write a more verbose signature:
    // `With<impl for<'cx> FnOnce(&mut Cx<'cx>) -> SomeConcreteReturnType>`.
    for<'cx> F: FnOnce(&mut Cx<'cx>) -> O;

impl<'cx, F, O> TryIntoJs<'cx> for With<F, O>
where
    F: FnOnce(&mut Cx) -> O,
    O: TryIntoJs<'cx>,
{
    type Value = O::Value;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        (self.0)(cx).try_into_js(cx)
    }
}

impl<F, O> super::private::Sealed for With<F, O> where for<'cx> F: FnOnce(&mut Cx<'cx>) -> O {}
