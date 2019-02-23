use neon::prelude::*;
use neon::object::This;
use neon::result::NeonResultExt;

fn add1(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let x = cx.argument::<JsNumber>(0)?.value();
    Ok(cx.number(x + 1.0))
}

pub fn return_js_function(mut cx: FunctionContext) -> NeonResult<&JsFunction> {
    JsFunction::new(&mut cx, add1)
}

pub fn call_js_function(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let f = cx.argument::<JsFunction>(0)?;
    let args: Vec<&JsNumber> = vec![cx.number(16.0)];
    let null = cx.null();
    f.call(&mut cx, null, args)?.downcast::<JsNumber>().or_throw(&mut cx)
}

pub fn construct_js_function(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let f = cx.argument::<JsFunction>(0)?;
    let zero = cx.number(0.0);
    let o = f.construct(&mut cx, vec![zero])?;
    let get_utc_full_year_method = o.get(&mut cx, "getUTCFullYear")?.downcast::<JsFunction>().or_throw(&mut cx)?;
    let args: Vec<&JsValue> = vec![];
    get_utc_full_year_method.call(&mut cx, o.upcast::<JsValue>(), args)?.downcast::<JsNumber>().or_throw(&mut cx)
}

trait CheckArgument<'a> {
    fn check_argument<V: Value>(&mut self, i: i32) -> NeonResult<&'a V>;
}

impl<'a, T: This> CheckArgument<'a> for CallContext<'a, T> {
    fn check_argument<V: Value>(&mut self, i: i32) -> NeonResult<&'a V> {
        self.argument::<V>(i)
    }
}

pub fn check_string_and_number(mut cx: FunctionContext) -> NeonResult<&JsUndefined> {
    cx.check_argument::<JsString>(0)?;
    cx.check_argument::<JsNumber>(1)?;
    Ok(cx.undefined())
}

pub fn panic(_: FunctionContext) -> NeonResult<&JsUndefined> {
    panic!("zomg")
}

pub fn panic_after_throw(mut cx: FunctionContext) -> NeonResult<&JsUndefined> {
    cx.throw_range_error::<_, ()>("entering throw state with a RangeError").unwrap_err();
    panic!("this should override the RangeError")
}

pub fn num_arguments(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let n = cx.len();
    Ok(cx.number(n))
}

pub fn return_this(mut cx: FunctionContext) -> NeonResult<&JsValue> {
    Ok(cx.this().upcast())
}

pub fn require_object_this(mut cx: FunctionContext) -> NeonResult<&JsUndefined> {
    let this = cx.this();
    let this = this.downcast::<JsObject>().or_throw(&mut cx)?;
    let t = cx.boolean(true);
    this.set(&mut cx, "modified", t)?;
    Ok(cx.undefined())
}

pub fn is_argument_zero_some(mut cx: FunctionContext) -> NeonResult<&JsBoolean> {
    let b = cx.argument_opt(0).is_some();
    Ok(cx.boolean(b))
}

pub fn require_argument_zero_string(mut cx: FunctionContext) -> NeonResult<&JsString> {
    let s = cx.argument(0)?;
    Ok(s)
}

pub fn execute_scoped(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let mut i = 0;
    for _ in 1..100 {
        cx.execute_scoped(|mut cx| {
            let n = cx.number(1);
            i += n.value() as i32;
        });
    }
    Ok(cx.number(i))
}

pub fn compute_scoped(mut cx: FunctionContext) -> NeonResult<&JsNumber> {
    let mut i = cx.number(0);
    for _ in 1..100 {
        i = cx.compute_scoped(|mut cx| {
            let n = cx.number(1);
            Ok(cx.number((i.value() as i32) + (n.value() as i32)))
        })?;
    }
    Ok(i)
}
