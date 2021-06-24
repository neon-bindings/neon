use neon::prelude::*;

pub fn return_js_symbol_with_description(mut cx: FunctionContext) -> JsResult<JsSymbol> {
    Ok(cx.symbol("neon:description"))
}

pub fn return_js_symbol(mut cx: FunctionContext) -> JsResult<JsSymbol> {
    Ok(JsSymbol::new(&mut cx))
}

pub fn read_js_symbol_description(mut cx: FunctionContext) -> JsResult<JsValue> {
    let symbol: Handle<JsSymbol> = cx.argument(0)?;
    match symbol.description(&mut cx) {
        None => Ok(cx.undefined().upcast()),
        Some(s) => Ok(cx.string(s).upcast()),
    }
}

pub fn accept_and_return_js_symbol(mut cx: FunctionContext) -> JsResult<JsSymbol> {
    let sym: Handle<JsSymbol> = cx.argument(0)?;
    Ok(sym)
}
