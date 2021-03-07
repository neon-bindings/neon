use neon::prelude::*;

pub fn return_js_string(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

pub fn run_string_as_script(mut cx: FunctionContext) -> JsResult<JsValue> {
    let string_script = cx.argument::<JsString>(0)?;
    string_script.run_as_script(&mut cx)
}
