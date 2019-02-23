use neon::prelude::*;

pub fn return_js_array(mut cx: FunctionContext) -> NeonResult<&JsArray> {
    Ok(cx.empty_array())
}

pub fn return_js_array_with_number(mut cx: FunctionContext) -> NeonResult<&JsArray> {
    let array: &JsArray = JsArray::new(&mut cx, 1);
    let n = cx.number(9000.0);
    array.set(&mut cx, 0, n)?;
    Ok(array)
}

pub fn return_js_array_with_string(mut cx: FunctionContext) -> NeonResult<&JsArray> {
    let array: &JsArray = JsArray::new(&mut cx, 1);
    let s = cx.string("hello node");
    array.set(&mut cx, 0, s)?;
    Ok(array)
}
