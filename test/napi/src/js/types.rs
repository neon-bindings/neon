use neon::prelude::*;

pub fn is_string(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let result = val.is_a::<JsString, _>(&mut cx);
    Ok(cx.boolean(result))
}

pub fn is_array(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let result = val.is_a::<JsArray, _>(&mut cx);
    Ok(cx.boolean(result))
}

pub fn is_array_buffer(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let result = val.is_a::<JsArrayBuffer, _>(&mut cx);
    Ok(cx.boolean(result))
}

pub fn is_uint32_array(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let result = val.is_a::<JsTypedArray<u32>, _>(&mut cx);
    Ok(cx.boolean(result))
}

pub fn is_boolean(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let result = val.is_a::<JsBoolean, _>(&mut cx);
    Ok(cx.boolean(result))
}

pub fn is_buffer(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let result = val.is_a::<JsBuffer, _>(&mut cx);
    Ok(cx.boolean(result))
}

pub fn is_error(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let result = val.is_a::<JsError, _>(&mut cx);
    Ok(cx.boolean(result))
}

pub fn is_null(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let result = val.is_a::<JsNull, _>(&mut cx);
    Ok(cx.boolean(result))
}

pub fn is_number(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let result = val.is_a::<JsNumber, _>(&mut cx);
    Ok(cx.boolean(result))
}

pub fn is_object(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let result = val.is_a::<JsObject, _>(&mut cx);
    Ok(cx.boolean(result))
}

pub fn is_undefined(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let val: Handle<JsValue> = cx.argument(0)?;
    let is_string = val.is_a::<JsUndefined, _>(&mut cx);
    Ok(cx.boolean(is_string))
}

pub fn strict_equals(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let v1: Handle<JsValue> = cx.argument(0)?;
    let v2: Handle<JsValue> = cx.argument(1)?;
    let eq = v1.strict_equals(&mut cx, v2);
    Ok(cx.boolean(eq))
}
