use neon::vm::{Call, JsResult};
use neon::js::{JsNumber};
use neon::mem::Handle;

pub fn return_js_number(call: Call) -> JsResult<JsNumber> {
    Ok(JsNumber::new(call.scope, 9000_f64))
}

pub fn return_large_js_number(call: Call) -> JsResult<JsNumber> {
    Ok(JsNumber::new(call.scope, 4294967296_f64))
}

pub fn return_negative_js_number(call: Call) -> JsResult<JsNumber> {
    Ok(JsNumber::new(call.scope, -9000_f64))
}

pub fn return_float_js_number(call: Call) -> JsResult<JsNumber> {
    Ok(JsNumber::new(call.scope, 1.4747_f64))
}

pub fn return_negative_float_js_number(call: Call) -> JsResult<JsNumber> {
    Ok(JsNumber::new(call.scope, -1.4747_f64))
}

pub fn accept_and_return_js_number(call: Call) -> JsResult<JsNumber> {
    let number: Handle<JsNumber> = try!(try!(call.arguments.require(call.scope, 0)).check::<JsNumber>());
    Ok(number)
}

pub fn accept_and_return_large_js_number(call: Call) -> JsResult<JsNumber> {
    let number: Handle<JsNumber> = try!(try!(call.arguments.require(call.scope, 0)).check::<JsNumber>());
    Ok(number)
}

pub fn accept_and_return_float_js_number(call: Call) -> JsResult<JsNumber> {
    let number: Handle<JsNumber> = try!(try!(call.arguments.require(call.scope, 0)).check::<JsNumber>());
    Ok(number)
}

pub fn accept_and_return_negative_js_number(call: Call) -> JsResult<JsNumber> {
    let number: Handle<JsNumber> = try!(try!(call.arguments.require(call.scope, 0)).check::<JsNumber>());
    Ok(number)
}
