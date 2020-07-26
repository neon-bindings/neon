use neon::prelude::*;

pub fn return_js_string(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}
