use std::sync::Mutex;

use neon::prelude::*;
use once_cell::sync::{Lazy, OnceCell};

pub fn get_and_replace(mut cx: FunctionContext) -> JsResult<JsValue> {
    static OBJECT: Lazy<Mutex<Option<Root<JsObject>>>> = Lazy::new(Default::default);

    let mut global = OBJECT.lock().unwrap_or_else(|err| err.into_inner());
    let next = cx.argument::<JsObject>(0)?.root(&mut cx);
    let previous = global.replace(next);

    Ok(match previous {
        None => cx.undefined().upcast(),
        Some(previous) => previous.into_inner(&mut cx).upcast(),
    })
}

pub fn get_or_init(mut cx: FunctionContext) -> JsResult<JsObject> {
    static OBJECT: OnceCell<Root<JsObject>> = OnceCell::new();

    let o = OBJECT.get_or_try_init(|| {
        cx.argument::<JsFunction>(0)?
            .call_with(&cx)
            .apply::<JsObject, _>(&mut cx)
            .map(|v| v.root(&mut cx))
    })?;

    Ok(o.to_inner(&mut cx))
}

pub fn get_or_init_clone(mut cx: FunctionContext) -> JsResult<JsObject> {
    static OBJECT: OnceCell<Root<JsObject>> = OnceCell::new();

    let o = OBJECT.get_or_try_init(|| {
        cx.argument::<JsFunction>(0)?
            .call_with(&cx)
            .apply::<JsObject, _>(&mut cx)
            .map(|v| v.root(&mut cx))
    })?;

    // Note: This intentionally uses `clone` instead of `to_inner` in order to
    // test the `clone` method.
    Ok(o.clone(&mut cx).into_inner(&mut cx))
}
