use neon::{prelude::*, reflect::eval};

pub fn return_js_string(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

pub fn return_js_string_utf16(mut cx: FunctionContext) -> JsResult<JsTypedArray<u16>> {
    let raw = "hello 🥹".encode_utf16().collect::<Vec<_>>();
    JsTypedArray::from_slice(&mut cx, &raw)
}

pub fn return_length_utf8(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let value = cx.argument::<JsString>(0)?.value(&mut cx);
    Ok(cx.number(value.len() as f64))
}

pub fn return_length_utf16(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let value = cx.argument::<JsString>(0)?.to_utf16(&mut cx);
    Ok(cx.number(value.len() as f64))
}

pub fn run_string_as_script(mut cx: FunctionContext) -> JsResult<JsValue> {
    let string_script = cx.argument::<JsString>(0)?;
    eval(&mut cx, string_script)
}
