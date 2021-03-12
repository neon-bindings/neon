use neon::prelude::*;

pub fn to_string(mut cx: FunctionContext) -> JsResult<JsString> {
    let arg: Handle<JsValue> = cx.argument(0)?;
    arg.to_string(&mut cx)
}
