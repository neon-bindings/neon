use std::future::Future;

use crate::{
    context::{Context, TaskContext},
    result::{JsResult, NeonResult},
    types::{extract::TryIntoJs, JsValue},
};

pub fn spawn<'cx, C, F, S>(cx: &mut C, fut: F, settle: S) -> JsResult<'cx, JsValue>
where
    C: Context<'cx>,
    F: Future + Send + 'static,
    F::Output: Send,
    S: FnOnce(TaskContext, F::Output) -> JsResult<JsValue> + Send + 'static,
{
    let rt = match crate::RUNTIME.get(cx) {
        Some(rt) => rt,
        None => return cx.throw_error("neon::RUNTIME is not initialized"),
    };

    let ch = cx.channel();
    let (d, promise) = cx.promise();

    rt.spawn(Box::pin(async move {
        let res = fut.await;
        let _ = d.try_settle_with(&ch, move |cx| settle(cx, res));
    }));

    Ok(promise.upcast())
}

pub trait ToNeonFutureMarker {
    type Marker;

    fn to_neon_future_marker(&self) -> Self::Marker;
}

impl<T, E> ToNeonFutureMarker for Result<T, E> {
    type Marker = NeonFutureMarkerResult;

    fn to_neon_future_marker(&self) -> Self::Marker {
        NeonFutureMarkerResult
    }
}

impl<T> ToNeonFutureMarker for &T {
    type Marker = NeonFutureMarkerValue;

    fn to_neon_future_marker(&self) -> Self::Marker {
        NeonFutureMarkerValue
    }
}

pub struct NeonFutureMarkerResult;
pub struct NeonFutureMarkerValue;

impl NeonFutureMarkerResult {
    pub fn make_result<'cx, C, T, E>(self, cx: &mut C, res: Result<T, E>) -> NeonResult<T>
    where
        C: Context<'cx>,
        E: TryIntoJs<'cx>,
    {
        res.or_else(|err| {
            let err = err.try_into_js(cx)?;
            cx.throw(err)
        })
    }
}

impl NeonFutureMarkerValue {
    pub fn make_result<'cx, C, T>(self, _cx: &mut C, res: T) -> NeonResult<T>
    where
        C: Context<'cx>,
    {
        Ok(res)
    }
}
