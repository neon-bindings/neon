use neon::object::This;
use neon::prelude::*;

fn add1(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let x = cx.argument::<JsNumber>(0)?.value(&mut cx);
    Ok(cx.number(x + 1.0))
}

pub fn return_js_function(mut cx: FunctionContext) -> JsResult<JsFunction> {
    JsFunction::new(&mut cx, add1)
}

pub fn call_js_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    cx.argument::<JsFunction>(0)?
        .call_with(&cx)
        .this(cx.null())
        .arg(cx.number(16.0))
        .apply(&mut cx)
}

fn get_math_max<'a>(cx: &mut FunctionContext<'a>) -> JsResult<'a, JsFunction> {
    let math = cx.global()
        .get(cx, "Math")?
        .downcast_or_throw::<JsObject, _>(cx)?;
    let max = math
        .get(cx, "max")?
        .downcast_or_throw::<JsFunction, _>(cx)?;
    Ok(max)
}

pub fn call_js_function_with_zero_args(mut cx: FunctionContext) -> JsResult<JsNumber> {
    get_math_max(&mut cx)?
        .call_with(&cx)
        .apply(&mut cx)
}

pub fn call_js_function_with_one_arg(mut cx: FunctionContext) -> JsResult<JsNumber> {
    get_math_max(&mut cx)?
        .call_with(&cx)
        .arg(cx.number(1.0))
        .apply(&mut cx)
}

pub fn call_js_function_with_two_args(mut cx: FunctionContext) -> JsResult<JsNumber> {
    get_math_max(&mut cx)?
        .call_with(&cx)
        .arg(cx.number(1.0))
        .arg(cx.number(2.0))
        .apply(&mut cx)
}

pub fn call_js_function_with_three_args(mut cx: FunctionContext) -> JsResult<JsNumber> {
    get_math_max(&mut cx)?
        .call_with(&cx)
        .arg(cx.number(1.0))
        .arg(cx.number(2.0))
        .arg(cx.number(3.0))
        .apply(&mut cx)
}

pub fn call_js_function_with_four_args(mut cx: FunctionContext) -> JsResult<JsNumber> {
    get_math_max(&mut cx)?
        .call_with(&cx)
        .arg(cx.number(1.0))
        .arg(cx.number(2.0))
        .arg(cx.number(3.0))
        .arg(cx.number(4.0))
        .apply(&mut cx)
}

pub fn call_js_function_with_custom_this(mut cx: FunctionContext) -> JsResult<JsObject> {
    let custom_this = cx.empty_object();
    let secret = cx.number(42.0);
    custom_this.set(&mut cx, "secret", secret)?;
    cx.argument::<JsFunction>(0)?
        .call_with(&cx)
        .this(custom_this)
        .apply(&mut cx)
}

pub fn call_js_function_with_heterogeneous_tuple(mut cx: FunctionContext) -> JsResult<JsArray> {
    cx.global()
        .get(&mut cx, "Array")?
        .downcast_or_throw::<JsFunction, _>(&mut cx)?
        .call_with(&cx)
        .args((cx.number(1.0), cx.string("hello"), cx.boolean(true)))
        .apply(&mut cx)
}

pub fn construct_js_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let o = cx
        .argument::<JsFunction>(0)?
        .construct_with(&cx)
        .arg(cx.number(0.0))
        .apply(&mut cx)?;
    let get_utc_full_year_method = o
        .get(&mut cx, "getUTCFullYear")?
        .downcast::<JsFunction, _>(&mut cx)
        .or_throw(&mut cx)?;
    get_utc_full_year_method
        .call_with(&cx)
        .this(o)
        .apply(&mut cx)
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
    cx.throw_range_error::<_, ()>("entering throw state with a RangeError")
        .unwrap_err();
    panic!("this should override the RangeError")
}

pub fn num_arguments(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let n = cx.len();
    Ok(cx.number(n))
}

pub fn return_this(mut cx: FunctionContext) -> JsResult<JsValue> {
    Ok(cx.this().upcast())
}

pub fn require_object_this(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let this = cx.this();
    let this = this.downcast::<JsObject, _>(&mut cx).or_throw(&mut cx)?;
    let t = cx.boolean(true);
    this.set(&mut cx, "modified", t)?;
    Ok(cx.undefined())
}

pub fn is_argument_zero_some(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let b = cx.argument_opt(0).is_some();
    Ok(cx.boolean(b))
}

pub fn require_argument_zero_string(mut cx: FunctionContext) -> JsResult<JsString> {
    let s = cx.argument(0)?;
    Ok(s)
}

pub fn execute_scoped(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let mut i = 0;
    for _ in 1..100 {
        cx.execute_scoped(|mut cx| {
            let n = cx.number(1);
            i += n.value(&mut cx) as i32;
        });
    }
    Ok(cx.number(i))
}

pub fn compute_scoped(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let mut i = cx.number(0);
    for _ in 1..100 {
        i = cx.compute_scoped(|mut cx| {
            let n = cx.number(1);
            let left = i.value(&mut cx) as i32;
            let right = n.value(&mut cx) as i32;
            Ok(cx.number(left + right))
        })?;
    }
    Ok(i)
}

pub fn throw_and_catch(mut cx: FunctionContext) -> JsResult<JsValue> {
    let v = cx
        .argument_opt(0)
        .unwrap_or_else(|| cx.undefined().upcast());

    cx.try_catch(|cx| cx.throw(v))
        .map(|_: ()| Ok(cx.string("unreachable").upcast()))
        .unwrap_or_else(Ok)
}

pub fn call_and_catch(mut cx: FunctionContext) -> JsResult<JsValue> {
    let f: Handle<JsFunction> = cx.argument(0)?;
    Ok(cx
        .try_catch(|cx| f.call_with(cx).this(cx.global()).apply(cx))
        .unwrap_or_else(|err| err))
}

pub fn get_number_or_default(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let n = cx
        .try_catch(|cx| Ok(cx.argument::<JsNumber>(0)?.value(cx)))
        .unwrap_or(0.0);

    Ok(cx.number(n))
}

pub fn is_construct(mut cx: FunctionContext) -> JsResult<JsObject> {
    let this = cx.this();
    let construct = matches!(cx.kind(), CallKind::Construct);
    let construct = cx.boolean(construct);
    this.set(&mut cx, "wasConstructed", construct)?;
    Ok(this)
}
