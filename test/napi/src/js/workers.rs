use std::{convert::TryFrom, sync::Mutex, thread, time::Duration};

use once_cell::sync::{Lazy, OnceCell};

use neon::prelude::*;
use neon::thread::LocalKey;

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

static THREAD_ID: LocalKey<u32> = LocalKey::new();

pub fn get_or_init_thread_id(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let id = cx.argument::<JsNumber>(0)?.value(&mut cx) as u32;
    let id: &u32 = THREAD_ID.get_or_init(&mut cx, || id);
    Ok(cx.number(*id))
}

static REENTRANT_LOCAL: LocalKey<u32> = LocalKey::new();

pub fn reentrant_try_init(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let f = cx.argument::<JsFunction>(0)?;
    let n = REENTRANT_LOCAL.get_or_try_init(&mut cx, |cx| {
        f.call_with(cx).exec(cx)?;
        Ok(42)
    })?;
    Ok(cx.number(*n))
}

pub fn get_reentrant_value(mut cx: FunctionContext) -> JsResult<JsValue> {
    let value = REENTRANT_LOCAL.get(&mut cx).cloned();
    match value {
        Some(n) => Ok(cx.number(n).upcast()),
        None => Ok(cx.null().upcast()),
    }
}

static GLOBAL_OBJECT: LocalKey<Root<JsObject>> = LocalKey::new();

pub fn stash_global_object(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    GLOBAL_OBJECT.get_or_try_init(&mut cx, |cx| {
        let global = cx.global();
        let global: Root<JsObject> = Root::new(cx, &global);
        Ok(global)
    })?;
    Ok(cx.undefined())
}

pub fn unstash_global_object(mut cx: FunctionContext) -> JsResult<JsValue> {
    Ok(match GLOBAL_OBJECT.get(&mut cx) {
        Some(root) => root.to_inner(&mut cx).upcast(),
        None => cx.null().upcast(),
    })
}

pub fn reject_after(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let err = cx.argument::<JsObject>(0)?.root(&mut cx);
    let ms = cx.argument::<JsNumber>(1)?.value(&mut cx) as i64;
    let ms = u64::try_from(ms)
        .or_else(|err| cx.throw_error(err.to_string()))
        .map(Duration::from_millis)?;

    let promise =
        cx.task(move || thread::sleep(ms))
            .promise(move |mut cx, _| -> JsResult<JsValue> {
                let err = err.into_inner(&mut cx);

                cx.throw(err)
            });

    Ok(promise)
}

pub struct Channels {
    _channel_1: Channel,
    _channel_2: Channel,
}
impl Finalize for Channels {}

pub fn box_channels(mut cx: FunctionContext) -> JsResult<JsBox<Channels>> {
    let channels = Channels {
        _channel_1: cx.channel(),
        _channel_2: cx.channel(),
    };

    Ok(cx.boxed(channels))
}
