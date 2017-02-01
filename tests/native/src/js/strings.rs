use neon::vm::{Call, JsResult};
use neon::js::{JsString, JsNumber};
use neon::mem::Handle;

pub fn return_js_string(call: Call) -> JsResult<JsString> {
    Ok(JsString::new(call.scope, "hello node").unwrap())
}

pub fn index_into_js_string(call: Call) -> JsResult<JsString> {
    let js_string = try!(try!(call.arguments.require(call.scope, 0)).check::<JsString>());
    let js_index = try!(try!(call.arguments.require(call.scope, 1)).check::<JsNumber>());
    let string = js_string.ucs2_value();
    let index = js_index.value() as usize;
    Ok(JsString::new_from_ucs2(call.scope, &string[index..index + 1]).unwrap())
}
