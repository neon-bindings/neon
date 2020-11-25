use neon::prelude::*;

pub fn to_string(mut cx: FunctionContext) -> JsResult<JsString> {
    let arg: Handle<JsValue> = cx.argument(0)?;
    Ok(arg.to_string(&mut cx)?)
}
