use neon::vm::{FunctionContext, JsResult, Context};
use neon::js::JsString;

pub fn return_js_string(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}
