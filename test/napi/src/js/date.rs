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

pub fn try_new_date(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let date_overflow = JsDate::try_new(&mut cx, JsDate::MAX_VALUE + 1.0);
    let date_underflow = JsDate::try_new(&mut cx, JsDate::MIN_VALUE - 1.0);
    assert_eq!(date_overflow.unwrap_err(), neon::types::DateErrorKind::Overflow);
    assert_eq!(date_underflow.unwrap_err(), neon::types::DateErrorKind::Underflow);
    Ok(cx.undefined())
}

pub fn check_date_is_invalid(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let time = JsDate::MIN_VALUE - 1.0;
    let date = cx.date(time);
    let is_valid = date.is_valid(&mut cx);
    let val = date.value(&mut cx);
    Ok(cx.boolean(is_valid))
}

pub fn create_and_get_invalid_date(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let time = JsDate::MAX_VALUE + 1.0;
    let date = cx.date(time).value(&mut cx);
    assert!(!cx.date(time).is_valid(&mut cx));
    assert!(cx.date(time).value(&mut cx).is_nan());
    Ok(cx.number(date))
}

pub fn get_date_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let date = cx.date(31415);
    let value = date.value(&mut cx);
    println!("{:?}", value);
    Ok(cx.number(value))
}
