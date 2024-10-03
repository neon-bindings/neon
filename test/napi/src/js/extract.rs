use either::Either;
use neon::{prelude::*, types::extract::*};

pub fn extract_values(mut cx: FunctionContext) -> JsResult<JsArray> {
    #[allow(clippy::type_complexity)]
    let (
        boolean,
        number,
        unit,
        string,
        Date(date),
        value,
        array_buf,
        buf,
        view,
        opt_number,
        opt_string,
    ): (
        bool,
        f64,
        (),
        String,
        Date,
        Handle<JsValue>,
        ArrayBuffer,
        Vec<u8>,
        Buffer,
        Option<f64>,
        Option<String>,
    ) = cx.args()?;

    let values = [
        boolean.try_into_js(&mut cx)?.upcast(),
        number.try_into_js(&mut cx)?.upcast(),
        unit.try_into_js(&mut cx)?.upcast(),
        string.try_into_js(&mut cx)?.upcast(),
        Date(date).try_into_js(&mut cx)?.upcast(),
        value,
        array_buf.try_into_js(&mut cx)?.upcast(),
        buf.try_into_js(&mut cx)?.upcast(),
        view.try_into_js(&mut cx)?.upcast(),
        opt_number
            .map(|n| cx.number(n).upcast::<JsValue>())
            .unwrap_or_else(|| cx.undefined().upcast()),
        opt_string
            .map(|n| cx.string(n).upcast::<JsValue>())
            .unwrap_or_else(|| cx.undefined().upcast()),
    ];

    let arr = cx.empty_array();

    for (i, v) in values.into_iter().enumerate() {
        arr.set(&mut cx, i as u32, v)?;
    }

    Ok(arr)
}

pub fn extract_buffer_sum(mut cx: FunctionContext) -> JsResult<JsNumber> {
    fn sum<'cx, T>(
        cx: &mut FunctionContext<'cx>,
        buf: Vec<T>,
        map: impl Fn(T) -> f64,
    ) -> JsResult<'cx, JsNumber> {
        Ok(cx.number(buf.into_iter().map(map).sum::<f64>()))
    }

    // `Float32Array`
    if let Some(buf) = cx.args_opt::<Vec<f32>>()? {
        return sum(&mut cx, buf, |n| n.into());
    }

    // `Float32Array`
    if let Some(buf) = cx.args_opt::<Vec<f64>>()? {
        return sum(&mut cx, buf, |n| n);
    }

    // `Buffer`
    if let Some(Buffer(buf)) = cx.args_opt()? {
        return sum(&mut cx, buf, |n| n as f64);
    }

    // `ArrayBuffer`
    if let Some(ArrayBuffer(buf)) = cx.args_opt()? {
        return sum(&mut cx, buf, |n| n as f64);
    }

    // `Uint8Array`
    if let Some(buf) = cx.args_opt::<Vec<u8>>()? {
        return sum(&mut cx, buf, |n| n as f64);
    }

    // `Uint16Array`
    if let Some(buf) = cx.args_opt::<Vec<u16>>()? {
        return sum(&mut cx, buf, |n| n as f64);
    }

    // `Uint32Array`
    if let Some(buf) = cx.args_opt::<Vec<u32>>()? {
        return sum(&mut cx, buf, |n| n as f64);
    }

    // `Uint64Array`
    if let Some(buf) = cx.args_opt::<Vec<u64>>()? {
        return sum(&mut cx, buf, |n| n as f64);
    }

    // `Int8Array`
    if let Some(buf) = cx.args_opt::<Vec<i8>>()? {
        return sum(&mut cx, buf, |n| n as f64);
    }

    // `Int16Array`
    if let Some(buf) = cx.args_opt::<Vec<i16>>()? {
        return sum(&mut cx, buf, |n| n as f64);
    }

    // `Int32Array`
    if let Some(buf) = cx.args_opt::<Vec<i32>>()? {
        return sum(&mut cx, buf, |n| n as f64);
    }

    // `Int64Array`
    let buf: Vec<i64> = cx.args()?;

    sum(&mut cx, buf, |n| n as f64)
}

pub fn extract_json_sum(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let Json::<Vec<f64>>(nums) = cx.args()?;

    Ok(cx.number(nums.into_iter().sum::<f64>()))
}

pub fn extract_single_add_one(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let n: f64 = cx.args()?;

    Ok(cx.number(n + 1.0))
}

#[neon::export]
pub fn extract_either(either: Either<String, f64>) -> String {
    match either {
        Either::Left(s) => format!("String: {s}"),
        Either::Right(n) => format!("Number: {n}"),
    }
}
