use neon::prelude::*;

#[neon::export]
fn export_noop() {}

fn manual_noop(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    Ok(cx.undefined())
}

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

fn call_callback_with_call(mut cx: FunctionContext) -> JsResult<JsValue> {
    let f = cx.argument::<JsFunction>(0)?;
    let s = cx.string("hello node");
    let n = cx.number(17.0);
    let b = cx.boolean(true);
    let this = cx.null();
    let args = vec![s.upcast(), n.upcast(), b.upcast()];
    f.call(&mut cx, this, args)
}

fn call_callback_with_call_with(mut cx: FunctionContext) -> JsResult<JsValue> {
    let f = cx.argument::<JsFunction>(0)?;
    f.call_with(&cx)
        .this(cx.null())
        .arg(cx.string("hello node"))
        .arg(cx.number(17.0))
        .arg(cx.boolean(true))
        .apply(&mut cx)
}

fn call_callback_with_bind(mut cx: FunctionContext) -> JsResult<JsValue> {
    let f = cx.argument::<JsFunction>(0)?;
    let this = cx.null();
    f.bind(&mut cx)
        .this(this)?
        .arg("hello node")?
        .arg(17.0)?
        .arg(true)?
        .call()
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // Export all macro-registered exports
    neon::registered().export(&mut cx)?;

    cx.export_function("hello", hello)?;
    cx.export_function("manualNoop", manual_noop)?;
    cx.export_function("callCallbackWithCall", call_callback_with_call)?;
    cx.export_function("callCallbackWithCallWith", call_callback_with_call_with)?;
    cx.export_function("callCallbackWithBind", call_callback_with_bind)?;

    Ok(())
}
