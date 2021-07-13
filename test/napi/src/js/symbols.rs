use neon::prelude::*;

pub fn return_js_symbol_from_context_helper(mut cx: FunctionContext) -> JsResult<JsSymbol> {
    Ok(cx.symbol("neon:context_helper"))
}

pub fn return_js_symbol_with_description(mut cx: FunctionContext) -> JsResult<JsSymbol> {
    let description: Handle<JsString> = cx.argument(0)?;
    Ok(JsSymbol::with_description(&mut cx, description))
}

pub fn return_js_symbol(mut cx: FunctionContext) -> JsResult<JsSymbol> {
    Ok(JsSymbol::new(&mut cx))
}

pub fn read_js_symbol_description(mut cx: FunctionContext) -> JsResult<JsValue> {
    let symbol: Handle<JsSymbol> = cx.argument(0)?;
    symbol
        .description(&mut cx)
        .map(|v| Ok(v.upcast()))
        .unwrap_or_else(|| Ok(cx.undefined().upcast()))
}

pub fn accept_and_return_js_symbol(mut cx: FunctionContext) -> JsResult<JsSymbol> {
    let sym: Handle<JsSymbol> = cx.argument(0)?;
    Ok(sym)
}
