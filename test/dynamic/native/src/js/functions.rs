use std::cell::RefCell;

use neon::object::This;
use neon::prelude::*;
use neon::result::Throw;

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
    f.call(&mut cx, null, args)?
        .downcast::<JsNumber>()
        .or_throw(&mut cx)
}

pub fn construct_js_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let f = cx.argument::<JsFunction>(0)?;
    let zero = cx.number(0.0);
    let o = f.construct(&mut cx, vec![zero])?;
    let get_utc_full_year_method = o
        .get(&mut cx, "getUTCFullYear")?
        .downcast::<JsFunction>()
        .or_throw(&mut cx)?;
    let args: Vec<Handle<JsValue>> = vec![];
    get_utc_full_year_method
        .call(&mut cx, o.upcast::<JsValue>(), args)?
        .downcast::<JsNumber>()
        .or_throw(&mut cx)
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
    let this = this.downcast::<JsObject>().or_throw(&mut cx)?;
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
            i += n.value() as i32;
        });
    }
    Ok(cx.number(i))
}

pub fn compute_scoped(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let mut i = cx.number(0);
    for _ in 1..100 {
        i = cx.compute_scoped(|mut cx| {
            let n = cx.number(1);
            Ok(cx.number((i.value() as i32) + (n.value() as i32)))
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
        .try_catch(|cx| {
            let global = cx.global();
            let args: Vec<Handle<JsValue>> = vec![];
            f.call(cx, global, args)
        })
        .unwrap_or_else(|err| err))
}

pub fn get_number_or_default(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let n = cx
        .try_catch(|cx| Ok(cx.argument::<JsNumber>(0)?.value()))
        .unwrap_or(0.0);

    Ok(cx.number(n))
}

pub fn panic_and_catch(mut cx: FunctionContext) -> JsResult<JsValue> {
    Ok(cx.try_catch(|_| panic!("oh no")).unwrap_or_else(|err| err))
}

thread_local! {
    static FORGED_THROW: RefCell<Option<Throw>> = RefCell::new(None);
}

fn forge_throw(mut cx: FunctionContext) -> JsResult<JsValue> {
    // Force a random JS error to temporarily enter the throwing state.
    let v: Handle<JsValue> = cx.null().upcast();
    match v.downcast_or_throw::<JsNumber, _>(&mut cx) {
        Ok(_) => panic!("failed to forge throw"),
        Err(throw) => {
            // Save the throw token in thread-local storage.
            FORGED_THROW.with(|forged_throw| {
                *forged_throw.borrow_mut() = Some(throw);
            })
        }
    }
    panic!("get us out of here");
}

pub fn unexpected_throw_and_catch(mut cx: FunctionContext) -> JsResult<JsValue> {
    // It's hard to forge a bogus throw token, but you can still do it by
    // forcing a real throw, saving the token for later, and catching the
    // throw to return the VM back out of its throwing state.
    let _ = cx.try_catch(|cx| {
        let forge: Handle<JsFunction> = JsFunction::new(cx, forge_throw)?;
        let null: Handle<JsValue> = cx.null().upcast();
        let args: Vec<Handle<JsValue>> = vec![];
        forge.call(cx, null, args)?;
        Ok(())
    });

    let retrieved_throw = FORGED_THROW.with(|forged_throw| forged_throw.borrow_mut().take());

    match retrieved_throw {
        Some(throw) => Ok(cx.try_catch(|_| Err(throw)).unwrap_or_else(|err| err)),
        None => Ok(cx.string("failed to retrieve forged throw").upcast()),
    }
}

pub fn downcast_error(mut cx: FunctionContext) -> JsResult<JsString> {
    let s = cx.string("hi");
    if let Err(e) = s.downcast::<JsNumber>() {
        Ok(cx.string(format!("{}", e)))
    } else {
        panic!()
    }
}
