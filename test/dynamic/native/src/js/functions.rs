use neon::vm::{FunctionContext, JsResult, JsResultExt, This, CallContext, Context};
use neon::mem::Handle;
use neon::js::{JsNumber, JsFunction, Object, JsValue, JsUndefined, JsString, Value};
use neon::js::error::{JsError, Kind};

fn add1(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let x = cx.argument::<JsNumber>(0)?.value();
    Ok(cx.number(x + 1.0))
}

pub fn return_js_function(mut cx: FunctionContext) -> JsResult<JsFunction> {
    JsFunction::new(&mut cx, add1)
}

pub fn call_js_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let f = cx.argument::<JsFunction>(0)?;
    let args: Vec<Handle<JsNumber>> = vec![cx.number(16.0)];
    let null = cx.null();
    f.call(&mut cx, null, args)?.downcast::<JsNumber>().unwrap_or_throw(&mut cx)
}

pub fn construct_js_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let f = cx.argument::<JsFunction>(0)?;
    let zero = cx.number(0.0);
    let o = f.construct(&mut cx, vec![zero])?;
    let get_utc_full_year_method = o.get(&mut cx, "getUTCFullYear")?.downcast::<JsFunction>().unwrap_or_throw(&mut cx)?;
    let args: Vec<Handle<JsValue>> = vec![];
    get_utc_full_year_method.call(&mut cx, o.upcast::<JsValue>(), args)?.downcast::<JsNumber>().unwrap_or_throw(&mut cx)
}

trait CheckArgument<'a> {
    fn check_argument<V: Value>(&mut self, i: i32) -> JsResult<'a, V>;
}

impl<'a, T: This> CheckArgument<'a> for CallContext<'a, T> {
    fn check_argument<V: Value>(&mut self, i: i32) -> JsResult<'a, V> {
        self.argument::<V>(i)
    }
}

pub fn check_string_and_number(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    cx.check_argument::<JsString>(0)?;
    cx.check_argument::<JsNumber>(1)?;
    Ok(cx.undefined())
}

pub fn panic(_: FunctionContext) -> JsResult<JsUndefined> {
    panic!("zomg")
}

pub fn panic_after_throw(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    JsError::throw::<_, ()>(&mut cx, Kind::RangeError, "entering throw state with a RangeError").unwrap_err();
    panic!("this should override the RangeError")
}
