use neon::vm::{Call, JsResult};
use neon::js::JsInteger;

pub fn return_js_integer(call: Call) -> JsResult<JsInteger> {
    Ok(JsInteger::new(call.scope, 9000))
}
