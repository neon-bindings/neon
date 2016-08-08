use neon::vm::{Call, JsResult, This, FunctionCall};
use neon::mem::Handle;
use neon::js::{JsNumber, JsNull, JsFunction, Object, JsValue, JsUndefined, JsString, Value};

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
    try!(f.call(scope, JsNull::new(), args)).check::<JsNumber>()
}

pub fn construct_js_function(call: Call) -> JsResult<JsNumber> {
    let scope = call.scope;
    let f = try!(try!(call.arguments.require(scope, 0)).check::<JsFunction>());
    let zero = JsNumber::new(scope, 0.0);
    let o = try!(f.construct(scope, vec![zero]));
    let get_utc_full_year_method = try!(try!(o.get(scope, "getUTCFullYear")).check::<JsFunction>());
    let args: Vec<Handle<JsValue>> = vec![];
    try!(get_utc_full_year_method.call(scope, o.upcast::<JsValue>(), args)).check::<JsNumber>()
}

trait CheckArgument<'a> {
    fn check_argument<V: Value>(&mut self, i: i32) -> JsResult<'a, V>;
}

impl<'a, T: This> CheckArgument<'a> for FunctionCall<'a, T> {
    fn check_argument<V: Value>(&mut self, i: i32) -> JsResult<'a, V> {
        try!(self.arguments.require(self.scope, i)).check::<V>()
    }
}

pub fn check_string_and_number(mut call: Call) -> JsResult<JsUndefined> {
    let x = try!(call.check_argument::<JsString>(0));
    let y = try!(call.check_argument::<JsNumber>(1));
    Ok(JsUndefined::new())
}
