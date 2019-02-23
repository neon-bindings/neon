use neon::prelude::*;

pub fn return_js_string(mut cx: FunctionContext) -> NeonResult<&JsString> {
    Ok(cx.string("hello node"))
}
