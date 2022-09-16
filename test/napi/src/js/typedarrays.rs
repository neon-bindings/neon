use neon::{
    prelude::*,
    types::buffer::{Binary, BorrowError, TypedArray},
};

pub fn return_array_buffer(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let b: Handle<JsArrayBuffer> = cx.array_buffer(16)?;
    Ok(b)
}

pub fn return_array_buffer_from_slice(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let len: Handle<JsNumber> = cx.argument(0)?;
    let len: f64 = len.value(&mut cx);
    let len: u32 = len as u32;
    let mut v: Vec<u8> = Vec::new();
    for i in 0..len {
        v.push(i as u8);
    }
    let b: Handle<JsArrayBuffer> = JsArrayBuffer::from_slice(&mut cx, &v)?;
    Ok(b)
}

pub fn read_array_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsTypedArray<u32>>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let lock = cx.lock();
    let n = buf.try_borrow(&lock).map(|buf| buf[i]).or_throw(&mut cx)?;

    Ok(cx.number(n))
}

pub fn read_array_buffer_with_borrow(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsArrayBuffer>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = buf.as_slice(&cx)[i];

    Ok(cx.number(n as f64))
}

pub fn write_array_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut b: Handle<JsArrayBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32 as usize;
    let x = cx.argument::<JsNumber>(2)?.value(&mut cx) as u8;
    let lock = cx.lock();

    b.try_borrow_mut(&lock)
        .map(|mut slice| {
            slice[i] = x;
        })
        .or_throw(&mut cx)?;

    Ok(cx.undefined())
}

pub fn write_array_buffer_with_borrow_mut(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut buf = cx.argument::<JsArrayBuffer>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = cx.argument::<JsNumber>(2)?.value(&mut cx) as u8;

    buf.as_mut_slice(&mut cx)[i] = n;

    Ok(cx.undefined())
}

pub fn read_typed_array_with_borrow(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsTypedArray<i32>>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = buf.as_slice(&cx)[i];

    Ok(cx.number(n as f64))
}

pub fn write_typed_array_with_borrow_mut(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut buf = cx.argument::<JsTypedArray<i32>>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = cx.argument::<JsNumber>(2)?.value(&mut cx) as i32;

    buf.as_mut_slice(&mut cx)[i] = n;

    Ok(cx.undefined())
}

pub fn read_u8_typed_array(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsTypedArray<u8>>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = buf.as_slice(&cx)[i];

    Ok(cx.number(n as f64))
}

pub fn copy_typed_array(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let source = cx.argument::<JsTypedArray<u32>>(0)?;
    let mut dest = cx.argument::<JsTypedArray<u32>>(1)?;
    let mut run = || -> Result<_, BorrowError> {
        let lock = cx.lock();
        let source = source.try_borrow(&lock)?;
        let mut dest = dest.try_borrow_mut(&lock)?;

        dest.copy_from_slice(&source);

        Ok(())
    };

    run().or_throw(&mut cx)?;

    Ok(cx.undefined())
}

pub fn return_uninitialized_buffer(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let b: Handle<JsBuffer> = unsafe { JsBuffer::uninitialized(&mut cx, 16)? };
    Ok(b)
}

pub fn return_buffer(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let b: Handle<JsBuffer> = cx.buffer(16)?;
    Ok(b)
}

pub fn return_external_buffer(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let data = cx.argument::<JsString>(0)?.value(&mut cx);
    let buf = JsBuffer::external(&mut cx, data.into_bytes());

    Ok(buf)
}

pub fn return_external_array_buffer(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let data = cx.argument::<JsString>(0)?.value(&mut cx);
    let buf = JsArrayBuffer::external(&mut cx, data.into_bytes());

    Ok(buf)
}

pub fn return_int8array_from_arraybuffer(mut cx: FunctionContext) -> JsResult<JsInt8Array> {
    let buf = cx.argument::<JsArrayBuffer>(0)?;
    JsInt8Array::from_buffer(&mut cx, buf)
}

pub fn return_int16array_from_arraybuffer(mut cx: FunctionContext) -> JsResult<JsInt16Array> {
    let buf = cx.argument::<JsArrayBuffer>(0)?;
    JsInt16Array::from_buffer(&mut cx, buf)
}

pub fn return_uint32array_from_arraybuffer(mut cx: FunctionContext) -> JsResult<JsUint32Array> {
    let buf = cx.argument::<JsArrayBuffer>(0)?;
    JsUint32Array::from_buffer(&mut cx, buf)
}

pub fn return_float64array_from_arraybuffer(mut cx: FunctionContext) -> JsResult<JsFloat64Array> {
    let buf = cx.argument::<JsArrayBuffer>(0)?;
    JsFloat64Array::from_buffer(&mut cx, buf)
}

pub fn return_biguint64array_from_arraybuffer(
    mut cx: FunctionContext,
) -> JsResult<JsBigUint64Array> {
    let buf = cx.argument::<JsArrayBuffer>(0)?;
    JsBigUint64Array::from_buffer(&mut cx, buf)
}

pub fn return_new_int32array(mut cx: FunctionContext) -> JsResult<JsInt32Array> {
    let len = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
    JsInt32Array::new(&mut cx, len)
}

pub fn return_int32array_from_slice(mut cx: FunctionContext) -> JsResult<JsInt32Array> {
    let len: Handle<JsNumber> = cx.argument(0)?;
    let len: f64 = len.value(&mut cx);
    let len: u32 = len as u32;
    let mut v: Vec<i32> = Vec::new();
    for i in 0..len {
        v.push(i as i32);
    }
    let a: Handle<JsInt32Array> = JsInt32Array::from_slice(&mut cx, &v)?;
    Ok(a)
}

pub fn return_uint32array_from_arraybuffer_region(
    mut cx: FunctionContext,
) -> JsResult<JsUint32Array> {
    let buf = cx.argument::<JsArrayBuffer>(0)?;
    let offset = cx.argument::<JsNumber>(1)?;
    let offset = offset.value(&mut cx);
    let len = cx.argument::<JsNumber>(2)?;
    let len = len.value(&mut cx);
    JsUint32Array::from_region(&mut cx, &buf.region(offset as usize, len as usize))
}

pub fn get_arraybuffer_byte_length(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsArrayBuffer>(0)?;
    let size = buf.size(&mut cx);
    let n = cx.number(size as u32);
    Ok(n)
}

fn typed_array_info<'cx, C, T: Binary>(
    cx: &mut C,
    a: Handle<'cx, JsTypedArray<T>>,
) -> JsResult<'cx, JsObject>
where
    C: Context<'cx>,
{
    let offset = a.offset(cx);
    let offset = cx.number(offset as u32);

    let len = a.len(cx);
    let len = cx.number(len as u32);

    let size = a.size(cx);
    let size = cx.number(size as u32);

    let buffer = a.buffer(cx);

    let obj = cx.empty_object();

    obj.set(cx, "byteOffset", offset)?;
    obj.set(cx, "length", len)?;
    obj.set(cx, "byteLength", size)?;
    obj.set(cx, "buffer", buffer)?;

    Ok(obj)
}

fn detach_and_then<'cx, F>(mut cx: FunctionContext<'cx>, f: F) -> JsResult<JsObject>
where
    F: FnOnce(
        &mut FunctionContext<'cx>,
        Handle<'cx, JsUint32Array>,
    ) -> NeonResult<Option<Handle<'cx, JsUint32Array>>>,
{
    let mut a = cx.argument::<JsUint32Array>(0)?;
    let detach = cx.argument::<JsFunction>(1)?;

    let before = typed_array_info(&mut cx, a)?;

    detach.call_with(&cx).arg(a).exec(&mut cx)?;

    if let Some(new_array) = f(&mut cx, a)? {
        a = new_array;
    }

    let after = typed_array_info(&mut cx, a)?;

    let result = cx.empty_object();

    result.set(&mut cx, "before", before)?;
    result.set(&mut cx, "after", after)?;

    Ok(result)
}

pub fn detach_same_handle(cx: FunctionContext) -> JsResult<JsObject> {
    detach_and_then(cx, |_, _| Ok(None))
}

pub fn detach_and_escape(cx: FunctionContext) -> JsResult<JsObject> {
    detach_and_then(cx, |cx, a| {
        let a = cx.compute_scoped(|_| Ok(a))?;
        Ok(Some(a))
    })
}

pub fn detach_and_cast(cx: FunctionContext) -> JsResult<JsObject> {
    detach_and_then(cx, |cx, a| {
        let v = a.upcast::<JsValue>();
        let a = v.downcast_or_throw::<JsUint32Array, _>(cx)?;
        Ok(Some(a))
    })
}

pub fn detach_and_unroot(cx: FunctionContext) -> JsResult<JsObject> {
    detach_and_then(cx, |cx, a| {
        let root = a.root(cx);
        let a = root.into_inner(cx);
        Ok(Some(a))
    })
}

pub fn get_typed_array_info(mut cx: FunctionContext) -> JsResult<JsObject> {
    let x = cx.argument::<JsValue>(0)?;

    if let Ok(a) = x.downcast::<JsTypedArray<u8>, _>(&mut cx) {
        typed_array_info(&mut cx, a)
    } else if let Ok(a) = x.downcast::<JsTypedArray<i8>, _>(&mut cx) {
        typed_array_info(&mut cx, a)
    } else if let Ok(a) = x.downcast::<JsTypedArray<u16>, _>(&mut cx) {
        typed_array_info(&mut cx, a)
    } else if let Ok(a) = x.downcast::<JsTypedArray<i16>, _>(&mut cx) {
        typed_array_info(&mut cx, a)
    } else if let Ok(a) = x.downcast::<JsTypedArray<u32>, _>(&mut cx) {
        typed_array_info(&mut cx, a)
    } else if let Ok(a) = x.downcast::<JsTypedArray<i32>, _>(&mut cx) {
        typed_array_info(&mut cx, a)
    } else if let Ok(a) = x.downcast::<JsTypedArray<u64>, _>(&mut cx) {
        typed_array_info(&mut cx, a)
    } else if let Ok(a) = x.downcast::<JsTypedArray<i64>, _>(&mut cx) {
        typed_array_info(&mut cx, a)
    } else if let Ok(a) = x.downcast::<JsTypedArray<f32>, _>(&mut cx) {
        typed_array_info(&mut cx, a)
    } else if let Ok(a) = x.downcast::<JsTypedArray<f64>, _>(&mut cx) {
        typed_array_info(&mut cx, a)
    } else {
        cx.throw_type_error("expected a typed array")
    }
}

pub fn build_f32_region(mut cx: FunctionContext) -> JsResult<JsValue> {
    let buf: Handle<JsArrayBuffer> = cx.argument(0)?;
    let offset: Handle<JsNumber> = cx.argument(1)?;
    let offset: usize = offset.value(&mut cx) as u32 as usize;
    let len: Handle<JsNumber> = cx.argument(2)?;
    let len: usize = len.value(&mut cx) as u32 as usize;
    let convert: Handle<JsBoolean> = cx.argument(3)?;
    let convert: bool = convert.value(&mut cx);

    let region = buf.region::<f32>(offset, len);

    if convert {
        let arr = region.to_typed_array(&mut cx)?;
        Ok(arr.upcast())
    } else {
        Ok(cx.undefined().upcast())
    }
}

pub fn build_f64_region(mut cx: FunctionContext) -> JsResult<JsValue> {
    let buf: Handle<JsArrayBuffer> = cx.argument(0)?;
    let offset: Handle<JsNumber> = cx.argument(1)?;
    let offset: usize = offset.value(&mut cx) as u32 as usize;
    let len: Handle<JsNumber> = cx.argument(2)?;
    let len: usize = len.value(&mut cx) as u32 as usize;
    let convert: Handle<JsBoolean> = cx.argument(3)?;
    let convert: bool = convert.value(&mut cx);

    let region = buf.region::<f64>(offset, len);

    if convert {
        let arr = region.to_typed_array(&mut cx)?;
        Ok(arr.upcast())
    } else {
        Ok(cx.undefined().upcast())
    }
}

pub fn read_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let b: Handle<JsBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let lock = cx.lock();
    let x = b
        .try_borrow(&lock)
        .map(|slice| slice[i])
        .or_throw(&mut cx)?;

    Ok(cx.number(x))
}

pub fn read_buffer_with_borrow(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsBuffer>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = buf.as_slice(&cx)[i];

    Ok(cx.number(n as f64))
}

pub fn write_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut b: Handle<JsBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let x = cx.argument::<JsNumber>(2)?.value(&mut cx) as u8;
    let lock = cx.lock();

    b.try_borrow_mut(&lock)
        .map(|mut slice| slice[i] = x)
        .or_throw(&mut cx)?;

    Ok(cx.undefined())
}

pub fn write_buffer_with_borrow_mut(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut buf = cx.argument::<JsBuffer>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = cx.argument::<JsNumber>(2)?.value(&mut cx) as u8;

    buf.as_mut_slice(&mut cx)[i] = n;

    Ok(cx.undefined())
}
