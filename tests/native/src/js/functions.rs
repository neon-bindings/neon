use neon::vm::{Call, JsResult};
use neon::mem::Handle;
use neon::js::{JsNumber, JsNull, JsFunction};

fn add1(call: Call) -> JsResult<JsNumber> {
    let scope = call.scope;
    let x = try!(try!(call.arguments.require(scope, 0)).check::<JsNumber>()).value();
    Ok(JsNumber::new(scope, x + 1.0))
}

pub fn return_js_function(call: Call) -> JsResult<JsFunction> {
    JsFunction::new(call.scope, add1)
}

pub fn call_js_function(call: Call) -> JsResult<JsNumber> {
    let scope = call.scope;
    let f = try!(try!(call.arguments.require(scope, 0)).check::<JsFunction>());
    let args: Vec<Handle<JsNumber>> = vec![JsNumber::new(scope, 16.0)];
    f.call(scope, JsNull::new(), args)
}
