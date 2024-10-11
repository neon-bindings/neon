use neon::prelude::*;

#[neon::export]
pub fn return_js_map<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsMap> {
    JsMap::new(cx)
}

#[neon::export]
pub fn return_js_map_with_number_as_keys_and_values<'cx>(
    cx: &mut FunctionContext<'cx>,
) -> JsResult<'cx, JsMap> {
    let map = JsMap::new(cx)?;
    {
        let key = cx.number(1);
        let val = cx.number(1000);
        map.set(cx, key, val)?;
    }
    {
        let key = cx.number(-1);
        let val = cx.number(-1000);
        map.set(cx, key, val)?;
    }
    Ok(map)
}

#[neon::export]
pub fn return_js_map_with_heterogeneous_keys_and_values<'cx>(
    cx: &mut FunctionContext<'cx>,
) -> JsResult<'cx, JsMap> {
    let map = JsMap::new(cx)?;
    {
        let key = cx.string("a");
        let val = cx.number(1);
        map.set(cx, key, val)?;
    }
    {
        let key = cx.number(26);
        let val = cx.string("z");
        map.set(cx, key, val)?;
    }
    Ok(map)
}

#[neon::export]
pub fn read_js_map<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsValue> {
    let map = cx.argument::<JsMap>(0)?;
    let key = cx.argument::<JsValue>(1)?;
    map.get(cx, key)
}

#[neon::export]
pub fn get_js_map_size<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsNumber> {
    let map = cx.argument::<JsMap>(0)?;
    map.size(cx).map(|x| cx.number(x))
}

#[neon::export]
pub fn modify_js_map<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsMap> {
    let map = cx.argument::<JsMap>(0)?;
    let key = cx.argument::<JsValue>(1)?;
    let value = cx.argument::<JsValue>(2)?;
    map.set(cx, key, value)
}

#[neon::export]
pub fn clear_js_map<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsUndefined> {
    let map = cx.argument::<JsMap>(0)?;
    map.clear(cx).map(|_| cx.undefined())
}

#[neon::export]
pub fn delete_js_map<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsBoolean> {
    let map = cx.argument::<JsMap>(0)?;
    let key = cx.argument::<JsValue>(1)?;
    map.delete(cx, key).map(|v| cx.boolean(v))
}

#[neon::export]
pub fn has_js_map<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsBoolean> {
    let map = cx.argument::<JsMap>(0)?;
    let key = cx.argument::<JsValue>(1)?;
    map.has(cx, key).map(|v| cx.boolean(v))
}

#[neon::export]
pub fn for_each_js_map<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsUndefined> {
    let map = cx.argument::<JsMap>(0)?;
    let cb: Handle<'_, JsValue> = cx.argument::<JsValue>(1)?;
    map.for_each(cx, cb)
}

#[neon::export]
pub fn group_by_js_map<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsMap> {
    let elements = cx.argument::<JsValue>(0)?;
    let cb = cx.argument::<JsValue>(1)?;
    JsMap::group_by(cx, elements, cb)
}
