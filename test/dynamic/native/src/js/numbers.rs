use neon::prelude::*;

pub fn return_js_number(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    Ok(cx.number(9000_f64))
}

pub fn return_large_js_number(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    Ok(cx.number(4294967296_f64))
}

pub fn return_negative_js_number(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    Ok(cx.number(-9000_f64))
}

pub fn return_float_js_number(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    Ok(cx.number(1.4747_f64))
}

pub fn return_negative_float_js_number(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    Ok(cx.number(-1.4747_f64))
}

pub fn accept_and_return_js_number(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let number: &JsNumber = cx.argument(0)?;
    Ok(number)
}

pub fn accept_and_return_large_js_number(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let number: &JsNumber = cx.argument(0)?;
    Ok(number)
}

pub fn accept_and_return_float_js_number(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let number: &JsNumber = cx.argument(0)?;
    Ok(number)
}

pub fn accept_and_return_negative_js_number(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let number: &JsNumber = cx.argument(0)?;
    Ok(number)
}
