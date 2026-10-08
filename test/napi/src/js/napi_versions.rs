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
