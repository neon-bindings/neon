use neon::prelude::*;
use neon::types::JsDate;

pub fn create_date(mut cx: FunctionContext) -> JsResult<JsDate> {
    let date = cx.date(31415);
    Ok(date)
}

pub fn create_date_from_value(mut cx: FunctionContext) -> JsResult<JsDate> {
    let time = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let date = cx.date(time);
    Ok(date)
}

pub fn check_date_is_valid(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let time = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let date = cx.date(time);
    let is_valid = date.is_valid(&mut cx);
    Ok(cx.boolean(is_valid))
}

pub fn check_date_is_invalid(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let time = 2f64.powf(64.0);
    let date = cx.date(time);
    let is_valid = date.is_valid(&mut cx);
    Ok(cx.boolean(is_valid))
}

pub fn get_date_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let date = cx.date(31415);
    let value = date.value(&mut cx);
    println!("{:?}", value);
    Ok(cx.number(value))
}
