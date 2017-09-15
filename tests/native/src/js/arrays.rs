use neon::vm::{Call, JsResult};
use neon::mem::Handle;
use neon::js::{JsInteger, JsString, JsArray, Object};

pub fn return_js_array(call: Call) -> JsResult<JsArray> {
    Ok(JsArray::new(call.scope, 0))
}

pub fn return_js_array_with_integer(call: Call) -> JsResult<JsArray> {
    let scope = call.scope;
    let array: Handle<JsArray> = JsArray::new(scope, 1);
    array.set(0, JsInteger::new(scope, 9000))?;
    Ok(array)
}

pub fn return_js_array_with_string(call: Call) -> JsResult<JsArray> {
    let scope = call.scope;
    let array: Handle<JsArray> = JsArray::new(scope, 1);
    array.set(0, JsString::new(scope, "hello node").unwrap())?;
    Ok(array)
}
