use std::{ffi::CStr, ptr};

use neon::{prelude::*, sys::bindings as sys};

#[neon::export]
fn napi9_symbol_for<'cx>(cx: &mut Cx<'cx>, key: String) -> Handle<'cx, JsValue> {
    let mut out = ptr::null_mut();

    unsafe {
        sys::symbol_for(cx.to_raw(), key.as_ptr().cast(), key.len(), &mut out).unwrap();
        JsValue::from_raw(cx, out)
    }
}

#[neon::export]
fn napi9_module_file_name(cx: &mut Cx) -> String {
    let mut out = ptr::null();

    unsafe {
        sys::get_module_file_name(cx.to_raw(), &mut out).unwrap();
        CStr::from_ptr(out).to_string_lossy().into_owned()
    }
}

#[neon::export]
fn napi10_property_key<'cx>(
    cx: &mut Cx<'cx>,
    key: String,
    value: Handle<'cx, JsValue>,
) -> JsResult<'cx, JsObject> {
    let mut out = ptr::null_mut();
    let obj = cx.empty_object();

    let key = unsafe {
        sys::create_property_key_utf8(cx.to_raw(), key.as_ptr().cast(), key.len(), &mut out)
            .unwrap();
        JsString::from_raw(cx, out)
    };

    obj.prop(cx, key).set(value)?;

    Ok(obj)
}

#[neon::export]
fn property_key_roundtrip<'cx>(
    cx: &mut Cx<'cx>,
    keys: Handle<'cx, JsArray>,
    value: Handle<'cx, JsValue>,
) -> JsResult<'cx, JsObject> {
    let obj = cx.empty_object();

    for key in keys.to_vec(cx)? {
        let key = key.downcast_or_throw::<JsString, _>(cx)?.value(cx);
        let handle = cx.property_key(&key);

        obj.prop(cx, handle).set(value)?;

        let by_handle: Handle<JsValue> = obj.prop(cx, handle).get()?;
        let by_str: Handle<JsValue> = obj.prop(cx, key.as_str()).get()?;

        if !by_handle.strict_equals(cx, value) || !by_str.strict_equals(cx, value) {
            return cx.throw_error(format!("property {key:?} did not round-trip"));
        }
    }

    Ok(obj)
}

#[neon::export]
fn sum_by_property_key<'cx>(
    cx: &mut Cx<'cx>,
    key: String,
    objs: Handle<'cx, JsArray>,
) -> NeonResult<f64> {
    let key = cx.property_key(key);
    let mut sum = 0.0;

    for obj in objs.to_vec(cx)? {
        let obj = obj.downcast_or_throw::<JsObject, _>(cx)?;
        sum += obj.prop(cx, key).get::<f64>()?;
    }

    Ok(sum)
}
