use neon::prelude::*;

pub fn return_js_map(mut cx: FunctionContext) -> JsResult<JsMap> {
    JsMap::new(&mut cx)
}

pub fn return_js_map_with_number_as_keys_and_values(mut cx: FunctionContext) -> JsResult<JsMap> {
    let map = JsMap::new(&mut cx)?;
    {
        let key = cx.number(1);
        let val = cx.number(1000);
        map.set(&mut cx, key, val)?;
    }
    {
        let key = cx.number(-1);
        let val = cx.number(-1000);
        map.set(&mut cx, key, val)?;
    }
    Ok(map)
}

pub fn return_js_map_with_heterogeneous_keys_and_values(
    mut cx: FunctionContext,
) -> JsResult<JsMap> {
    let map = JsMap::new(&mut cx)?;
    {
        let key = cx.string("a");
        let val = cx.number(1);
        map.set(&mut cx, key, val)?;
    }
    {
        let key = cx.number(26);
        let val = cx.string("z");
        map.set(&mut cx, key, val)?;
    }
    Ok(map)
}

pub fn read_js_map(mut cx: FunctionContext) -> JsResult<JsValue> {
    let map = cx.argument::<JsMap>(0)?;
    let key = cx.argument::<JsValue>(1)?;
    map.get(&mut cx, key)
}

pub fn get_js_map_size(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let map = cx.argument::<JsMap>(0)?;
    map.size(&mut cx)
}

pub fn modify_js_map(mut cx: FunctionContext) -> JsResult<JsMap> {
    let map = cx.argument::<JsMap>(0)?;
    let key = cx.argument::<JsValue>(1)?;
    let value = cx.argument::<JsValue>(2)?;
    map.set(&mut cx, key, value)
}

pub fn clear_js_map(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let map = cx.argument::<JsMap>(0)?;
    map.clear(&mut cx)
}

pub fn delete_js_map(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let map = cx.argument::<JsMap>(0)?;
    let key = cx.argument::<JsValue>(1)?;
    map.delete(&mut cx, key).map(|v| cx.boolean(v))
}

pub fn has_js_map(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let map = cx.argument::<JsMap>(0)?;
    let key = cx.argument::<JsValue>(1)?;
    map.has(&mut cx, key).map(|v| cx.boolean(v))
}

pub fn for_each_js_map(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let map = cx.argument::<JsMap>(0)?;
    let cb: Handle<'_, JsValue> = cx.argument::<JsValue>(1)?;
    map.for_each(&mut cx, cb)
}

pub fn group_by_js_map(mut cx: FunctionContext) -> JsResult<JsMap> {
    let elements = cx.argument::<JsValue>(0)?;
    let cb = cx.argument::<JsValue>(1)?;
    JsMap::group_by(&mut cx, elements, cb)
}
