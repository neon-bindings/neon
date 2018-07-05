use neon::vm::{FunctionContext, JsResult, Context};
use neon::js::{JsNumber, JsInteger};
use neon::mem::Handle;

pub fn return_js_number(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(9000_f64))
}

pub fn return_large_js_number(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(4294967296_f64))
}

pub fn return_negative_js_number(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(-9000_f64))
}

pub fn return_float_js_number(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(1.4747_f64))
}

pub fn return_negative_float_js_number(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(-1.4747_f64))
}

// DEPRECATE(0.2)
pub fn return_js_integer(mut cx: FunctionContext) -> JsResult<JsInteger> {
    Ok(JsInteger::new(&mut cx, 17))
}

pub fn accept_and_return_js_number(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let number: Handle<JsNumber> = cx.argument(0)?;
    Ok(number)
}

pub fn accept_and_return_large_js_number(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let number: Handle<JsNumber> = cx.argument(0)?;
    Ok(number)
}

pub fn accept_and_return_float_js_number(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let number: Handle<JsNumber> = cx.argument(0)?;
    Ok(number)
}

pub fn accept_and_return_negative_js_number(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let number: Handle<JsNumber> = cx.argument(0)?;
    Ok(number)
}

// DEPRECATE(0.2)
pub fn accept_and_return_js_integer(mut cx: FunctionContext) -> JsResult<JsInteger> {
    let x: Handle<JsInteger> = cx.argument(0)?;
    Ok(x)
}
