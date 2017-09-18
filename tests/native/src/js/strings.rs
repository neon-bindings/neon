use neon::vm::{Call, JsResult};
use neon::js::JsString;

pub fn return_js_string(call: Call) -> JsResult<JsString> {
    Ok(JsString::new(call.scope, "hello node").unwrap())
}
