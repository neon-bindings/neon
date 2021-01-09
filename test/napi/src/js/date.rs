use neon::prelude::*;
use neon::types::JsDate;

pub fn create_date(mut cx: FunctionContext) -> JsResult<JsDate> {
    let date = JsDate::new_lossy(&mut cx, 31415);
    Ok(date)
}

pub fn create_date_from_value(mut cx: FunctionContext) -> JsResult<JsDate> {
    let time = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let date = JsDate::new_lossy(&mut cx, time);
    Ok(date)
}

pub fn check_date_is_valid(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let time = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let date = JsDate::new_lossy(&mut cx, time);
    let is_valid = date.is_valid(&mut cx);
    Ok(cx.boolean(is_valid))
}

pub fn try_new_date(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let _date_overflow = JsDate::new(&mut cx, JsDate::MAX_VALUE + 1.0);
    let _date_underflow = JsDate::new(&mut cx, JsDate::MIN_VALUE - 1.0);
    Ok(cx.undefined())
}

pub fn try_new_lossy_date(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let date_overflow = JsDate::new(&mut cx, JsDate::MAX_VALUE + 1.0);
    let date_underflow = JsDate::new(&mut cx, JsDate::MIN_VALUE - 1.0);
    assert_eq!(date_overflow.unwrap_err().kind(), neon::types::DateErrorKind::Overflow);
    assert_eq!(date_underflow.unwrap_err().kind(), neon::types::DateErrorKind::Underflow);
    Ok(cx.undefined())
}

pub fn check_date_is_invalid(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let time = JsDate::MIN_VALUE - 1.0;
    let date = JsDate::new_lossy(&mut cx, time);
    let is_valid = date.is_valid(&mut cx);
    let val = date.value(&mut cx);
    Ok(cx.boolean(is_valid))
}

pub fn create_and_get_invalid_date(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let time = JsDate::MAX_VALUE + 1.0;
    let date = JsDate::new_lossy(&mut cx, time).value(&mut cx);
    assert!(!JsDate::new_lossy(&mut cx, time).is_valid(&mut cx));
    assert!(JsDate::new_lossy(&mut cx, time).value(&mut cx).is_nan());

    let time = JsDate::MIN_VALUE - 1.0;
    let date = JsDate::new_lossy(&mut cx, time).value(&mut cx);
    assert!(!JsDate::new_lossy(&mut cx, time).is_valid(&mut cx));
    assert!(JsDate::new_lossy(&mut cx, time).value(&mut cx).is_nan());

    let time = JsDate::MAX_VALUE + 2.0;
    let date = JsDate::new_lossy(&mut cx, time).value(&mut cx);
    assert!(!JsDate::new_lossy(&mut cx, time).is_valid(&mut cx));
    assert!(JsDate::new_lossy(&mut cx, time).value(&mut cx).is_nan());

    let time = JsDate::MAX_VALUE + 3.0;
    let date = JsDate::new_lossy(&mut cx, time).value(&mut cx);
    assert!(!JsDate::new_lossy(&mut cx, time).is_valid(&mut cx));
    assert!(JsDate::new_lossy(&mut cx, time).value(&mut cx).is_nan());

    let time = JsDate::MAX_VALUE + 1_000.0;
    let date = JsDate::new_lossy(&mut cx, time).value(&mut cx);
    assert!(!JsDate::new_lossy(&mut cx, time).is_valid(&mut cx));
    assert!(JsDate::new_lossy(&mut cx, time).value(&mut cx).is_nan());

    Ok(cx.number(date))
}

pub fn get_date_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let date = JsDate::new_lossy(&mut cx, 31415);
    let value = date.value(&mut cx);
    println!("{:?}", value);
    Ok(cx.number(value))
}
