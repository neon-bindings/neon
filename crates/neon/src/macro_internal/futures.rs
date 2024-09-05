use std::future::Future;

use crate::{
    context::{Context, Cx, TaskContext},
    result::JsResult,
    types::JsValue,
};

pub fn spawn<'cx, F, S>(cx: &mut Cx<'cx>, fut: F, settle: S) -> JsResult<'cx, JsValue>
where
    F: Future + Send + 'static,
    F::Output: Send,
    S: FnOnce(TaskContext, F::Output) -> JsResult<JsValue> + Send + 'static,
{
    let rt = match crate::executor::RUNTIME.get(cx) {
        Some(rt) => rt,
        None => return cx.throw_error("must initialize with neon::set_global_executor"),
    };

    let ch = cx.channel();
    let (d, promise) = cx.promise();

    rt.spawn(Box::pin(async move {
        let res = fut.await;
        let _ = d.try_settle_with(&ch, move |cx| settle(cx, res));
    }));

    Ok(promise.upcast())
}
