use neon::prelude::*;

pub fn return_js_global_object(mut cx: FunctionContext) -> NeonResult<&JsObject> {
    Ok(cx.global())
}

pub fn return_js_object(mut cx: FunctionContext) -> NeonResult<&JsObject> {
    Ok(cx.empty_object())
}

pub fn return_js_object_with_mixed_content(mut cx: FunctionContext) -> NeonResult<&JsObject> {
    let js_object: &JsObject = cx.empty_object();
    let n = cx.number(9000.0);
    js_object.set(&mut cx, "number", n)?;
    let s = cx.string("hello node");
    js_object.set(&mut cx, "string", s)?;
    Ok(js_object)
}

pub fn return_js_object_with_number(mut cx: FunctionContext) -> NeonResult<&JsObject> {
    let js_object: &JsObject = cx.empty_object();
    let n = cx.number(9000.0);
    js_object.set(&mut cx, "number", n)?;
    Ok(js_object)
}

pub fn return_js_object_with_string(mut cx: FunctionContext) -> NeonResult<&JsObject> {
    let js_object: &JsObject = cx.empty_object();
    let s = cx.string("hello node");
    js_object.set(&mut cx, "string", s)?;
    Ok(js_object)
}

pub fn return_array_buffer(mut cx: FunctionContext) -> NeonResult<&JsArrayBuffer> {
    let b: &JsArrayBuffer = cx.array_buffer(16)?;
    Ok(b)
}

pub fn read_array_buffer_with_lock(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let b: &JsArrayBuffer = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value() as u32 as usize;
    let x = {
        let guard = cx.lock();
        let data = b.borrow(&guard);
        let slice = data.as_slice::<u32>();
        slice[i]
    };
    Ok(cx.number(x))
}

pub fn read_array_buffer_with_borrow(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let b: &JsArrayBuffer = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value() as u32 as usize;
    let x = cx.borrow(b, |data| { data.as_slice::<u32>()[i] });
    Ok(cx.number(x))
}

pub fn write_array_buffer_with_lock(mut cx: FunctionContext) -> NeonResult<&JsUndefined> {
    let b: &JsArrayBuffer = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value() as u32 as usize;
    let x = cx.argument::<JsNumber>(2)?.value() as u32;
    {
        let guard = cx.lock();
        let data = b.borrow_mut(&guard);
        let slice = data.as_mut_slice::<u32>();
        slice[i] = x;
    }
    Ok(cx.undefined())
}

pub fn write_array_buffer_with_borrow_mut(mut cx: FunctionContext) -> NeonResult<&JsUndefined> {
    let b: &JsArrayBuffer = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value() as u32 as usize;
    let x = cx.argument::<JsNumber>(2)?.value() as u32;
    cx.borrow_mut(b, |data| { data.as_mut_slice::<u32>()[i] = x; });
    Ok(cx.undefined())
}

pub fn read_buffer_with_lock(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let b: &JsBuffer = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value() as u32 as usize;
    let x = {
        let guard = cx.lock();
        let data = b.borrow(&guard);
        let slice = data.as_slice::<u32>();
        slice[i]
    };
    Ok(cx.number(x))
}

pub fn read_buffer_with_borrow(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let b: &JsBuffer = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value() as u32 as usize;
    let x = cx.borrow(b, |data| { data.as_slice::<u32>()[i] });
    Ok(cx.number(x))
}

pub fn write_buffer_with_lock(mut cx: FunctionContext) -> NeonResult<&JsUndefined> {
    let b: &JsBuffer = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value() as u32 as usize;
    let x = cx.argument::<JsNumber>(2)?.value() as u32;
    {
        let guard = cx.lock();
        let data = b.borrow_mut(&guard);
        let slice = data.as_mut_slice::<u32>();
        slice[i] = x;
    }
    Ok(cx.undefined())
}

pub fn write_buffer_with_borrow_mut(mut cx: FunctionContext) -> NeonResult<&JsUndefined> {
    let b: &JsBuffer = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value() as u32 as usize;
    let x = cx.argument::<JsNumber>(2)?.value() as u32;
    cx.borrow_mut(b, |data| { data.as_mut_slice::<u32>()[i] = x; });
    Ok(cx.undefined())
}
