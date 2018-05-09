use neon::vm::{Call, JsResult, This, FunctionCall};
use neon::mem::Handle;
use neon::js::{JsNumber, JsNull, JsFunction, Object, JsValue, JsUndefined, JsString, Value};
use neon::js::error::{JsError, Kind};

fn add1(call: Call) -> JsResult<JsNumber> {
    let scope = call.scope;
    let x = call.arguments.require(scope, 0)?.check::<JsNumber>()?.value();
    Ok(JsNumber::new(scope, x + 1.0))
}

pub fn return_js_function(call: Call) -> JsResult<JsFunction> {
    JsFunction::from_fn(call.scope, add1)
}

pub fn call_js_function(call: Call) -> JsResult<JsNumber> {
    let scope = call.scope;
    let f = call.arguments.require(scope, 0)?.check::<JsFunction>()?;
    let args: Vec<Handle<JsNumber>> = vec![JsNumber::new(scope, 16.0)];
    f.call(scope, JsNull::new(), args)?.check::<JsNumber>()
}

pub fn construct_js_function(call: Call) -> JsResult<JsNumber> {
    let scope = call.scope;
    let f = call.arguments.require(scope, 0)?.check::<JsFunction>()?;
    let zero = JsNumber::new(scope, 0.0);
    let o = f.construct(scope, vec![zero])?;
    let get_utc_full_year_method = o.get(scope, "getUTCFullYear")?.check::<JsFunction>()?;
    let args: Vec<Handle<JsValue>> = vec![];
    get_utc_full_year_method.call(scope, o.upcast::<JsValue>(), args)?.check::<JsNumber>()
}

pub fn return_js_closure(call: Call) -> JsResult<JsFunction> {
    let scope = call.scope;
    let x = call.arguments.require(scope, 0)?.check::<JsNumber>()?.value();
    JsFunction::new(scope, Box::new(move |inner| {
        let y = inner.arguments.require(inner.scope, 0)?.check::<JsNumber>()?.value();
        Ok(JsNumber::new(inner.scope, x + y))
    }))
}

pub fn return_js_mutable_closure(call: Call) -> JsResult<JsFunction> {
    let scope = call.scope;
    let mut x = 0;
    let mut y = 1;
    JsFunction::new(scope, Box::new(move |inner| {
        let sum = x + y;
        x = y;
        y = sum;
        Ok(JsNumber::new(inner.scope, sum as f64))
    }))
}

trait CheckArgument<'a> {
    fn check_argument<V: Value>(&mut self, i: i32) -> JsResult<'a, V>;
}

impl<'a, T: This> CheckArgument<'a> for FunctionCall<'a, T> {
    fn check_argument<V: Value>(&mut self, i: i32) -> JsResult<'a, V> {
        self.arguments.require(self.scope, i)?.check::<V>()
    }
}

pub fn check_string_and_number(mut call: Call) -> JsResult<JsUndefined> {
    call.check_argument::<JsString>(0)?;
    call.check_argument::<JsNumber>(1)?;
    Ok(JsUndefined::new())
}

pub fn panic(_: Call) -> JsResult<JsUndefined> {
    panic!("zomg")
}

pub fn panic_after_throw(_: Call) -> JsResult<JsUndefined> {
    JsError::throw::<()>(Kind::RangeError, "entering throw state with a RangeError").unwrap_err();
    panic!("this should override the RangeError")
}
