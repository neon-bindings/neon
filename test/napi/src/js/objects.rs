use std::borrow::Cow;

use neon::{prelude::*, types::buffer::TypedArray};

pub fn return_js_global_object(mut cx: FunctionContext) -> JsResult<JsObject> {
    Ok(cx.global_object())
}

pub fn return_js_object(mut cx: FunctionContext) -> JsResult<JsObject> {
    Ok(cx.empty_object())
}

pub fn return_js_object_with_mixed_content(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_object: Handle<JsObject> = cx.empty_object();
    let n = cx.number(9000.0);
    js_object.set(&mut cx, "number", n)?;
    let s = cx.string("hello node");
    js_object.set(&mut cx, "string", s)?;
    Ok(js_object)
}

pub fn return_js_object_with_number(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_object: Handle<JsObject> = cx.empty_object();
    let n = cx.number(9000.0);
    js_object.set(&mut cx, "number", n)?;
    Ok(js_object)
}

pub fn return_js_object_with_string(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_object: Handle<JsObject> = cx.empty_object();
    let s = cx.string("hello node");
    js_object.set(&mut cx, "string", s)?;
    Ok(js_object)
}

pub fn freeze_js_object(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let obj: Handle<JsObject> = cx.argument::<JsObject>(0)?;
    match obj.freeze(&mut cx) {
        Ok(_) => Ok(cx.undefined()),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

pub fn seal_js_object(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let obj: Handle<JsObject> = cx.argument::<JsObject>(0)?;
    match obj.seal(&mut cx) {
        Ok(_) => Ok(cx.undefined()),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

// Accepts either a `JsString` or `JsBuffer` and returns the contents as
// as bytes; avoids copying.
fn get_bytes<'cx, 'a, C>(cx: &'a mut C, v: Handle<JsValue>) -> NeonResult<Cow<'a, [u8]>>
where
    C: Context<'cx>,
{
    if let Ok(v) = v.downcast::<JsString, _>(cx) {
        return Ok(Cow::Owned(v.value(cx).into_bytes()));
    }

    if let Ok(v) = v.downcast::<JsBuffer, _>(cx) {
        return Ok(Cow::Borrowed(v.as_slice(cx)));
    }

    cx.throw_type_error("Value must be a string or Buffer")
}

pub fn byte_length(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let v = cx.argument::<JsValue>(0)?;
    let bytes = get_bytes(&mut cx, v)?;

    // `v` is dropped here, but `bytes` is still valid since the data is on the V8 heap

    let len = bytes.len();

    Ok(cx.number(len as f64))
}

pub fn call_nullary_method(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let obj: Handle<JsObject> = cx.argument::<JsObject>(0)?;
    obj.call_method_with(&mut cx, "nullary")?.apply(&mut cx)
}

pub fn call_unary_method(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let obj: Handle<JsObject> = cx.argument::<JsObject>(0)?;
    let x: Handle<JsNumber> = cx.argument::<JsNumber>(1)?;
    obj.call_method_with(&mut cx, "unary")?
        .arg(x)
        .apply(&mut cx)
}

pub fn call_symbol_method(mut cx: FunctionContext) -> JsResult<JsString> {
    let obj: Handle<JsObject> = cx.argument::<JsObject>(0)?;
    let sym: Handle<JsValue> = cx.argument::<JsValue>(1)?;
    obj.call_method_with(&mut cx, sym)?.apply(&mut cx)
}

pub fn get_property_with_prop(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let obj: Handle<JsObject> = cx.argument::<JsObject>(0)?;
    let n: f64 = obj.prop(&mut cx, "number").get()?;
    Ok(cx.number(n))
}

pub fn set_property_with_prop(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let obj: Handle<JsObject> = cx.argument::<JsObject>(0)?;
    let b = obj.prop(&mut cx, "number").set(42)?;
    Ok(cx.boolean(b))
}

pub fn call_methods_with_prop(mut cx: FunctionContext) -> JsResult<JsString> {
    let obj: Handle<JsObject> = cx.argument::<JsObject>(0)?;
    obj.prop(&mut cx, "setName")
        .bind()?
        .arg("Wonder Woman")?
        .apply()?;
    obj.prop(&mut cx, "toString").bind()?.apply()
}

pub fn call_non_method_with_prop(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let obj: Handle<JsObject> = cx.argument::<JsObject>(0)?;
    obj.prop(&mut cx, "number")
        .bind()?
        .apply()?;
    Ok(cx.undefined())
}
