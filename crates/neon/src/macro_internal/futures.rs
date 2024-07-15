use std::future::Future;

use crate::{
    context::{Context, TaskContext},
    result::JsResult,
    types::JsValue,
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
