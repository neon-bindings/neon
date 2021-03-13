use neon::prelude::*;
use neon::reflect::eval;

pub fn return_js_string(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

pub fn run_string_as_script(mut cx: FunctionContext) -> JsResult<JsValue> {
    let string_script = cx.argument::<JsString>(0)?;
    eval(&mut cx, string_script)
}
