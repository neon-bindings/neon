use once_cell::unsync::Lazy;
use once_cell::unsync::OnceCell;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::TransparentNoCopyWrapper;
use super::Value;
use crate::context::Context;
use crate::object::Object;
use crate::prelude::Handle;
use crate::result::JsResult;
use crate::result::NeonResult;
use crate::sys::Value__;
use crate::types::private::ValueInternal;
use crate::types::JsError;
use crate::types::JsFunction;
use crate::types::JsNumber;
use crate::types::JsObject;
use crate::types::JsValue;

static KEY_NEON_CACHE: &str = "__neon_cache";

thread_local! {
    // Symbol("__neon_cache")
    static CACHE_SYMBOL: OnceCell<*mut Value__> = OnceCell::default();
}

/// Reference counted JavaScript value with a static lifetime for use in async closures
pub struct RootGlobal<T> {
    pub(crate) count: Rc<RefCell<u32>>,
    pub(crate) inner: Rc<*mut Value__>,
    _p: PhantomData<T>,
}

impl<T: Value> RootGlobal<T> {
    pub(crate) fn new<'a>(
        cx: &mut impl Context<'a>,
        value: Handle<'a, T>,
    ) -> NeonResult<RootGlobal<T>> {
        Ok(Self {
            count: Rc::new(RefCell::new(1)),
            inner: Rc::new(set_ref(cx, value)?),
            _p: Default::default(),
        })
    }

    pub fn clone<'a>(&self, cx: impl Context<'a>) -> Result<RootGlobal<T>, ()> {
        todo!();
    }

    pub fn deref<'a>(&self, cx: &mut impl Context<'a>) -> JsResult<'a, T> {
        // TODO error handling
        let env_raw = cx.env();
        let hydrated = unsafe { T::from_local(env_raw, *self.inner) };
        Ok(Handle::new_internal(hydrated))
    }

    pub fn drop<'a>(&self, cx: impl Context<'a>) -> Result<(), ()> {
        todo!();
    }
}

/*
  globalThis = {
    [key: Symbol("__neon_cache")]: Set<any>
  }
*/
fn get_cache<'a>(cx: &mut impl Context<'a>) -> JsResult<'a, JsObject> {
    let global_this = cx.global_object();

    let neon_cache_symbol = CACHE_SYMBOL.with({
        |raw_value| {
            raw_value
                .get_or_try_init(|| -> NeonResult<*mut Value__> {
                    let symbol_ctor = global_this.get::<JsFunction, _, _>(cx, "Symbol")?;
                    let set_ctor = global_this.get::<JsFunction, _, _>(cx, "Set")?;

                    let neon_cache = set_ctor.construct(cx, &[])?;

                    let key = cx.string(KEY_NEON_CACHE);
                    let symbol: Handle<JsValue> = symbol_ctor.call_with(cx).arg(key).apply(cx)?;
                    let symbol_raw = symbol.to_local();

                    global_this.set(cx, symbol, neon_cache)?;

                    Ok(symbol_raw)
                })
                .cloned()
        }
    })?;

    let neon_cache_symbol =
        Handle::new_internal(unsafe { JsValue::from_local(cx.env(), neon_cache_symbol) });

    let Some(neon_cache) = global_this.get_opt::<JsObject, _, _>(cx, neon_cache_symbol)? else {
        return Err(cx.throw_error("Unable to find cache")?);
    };

    console_log(cx, &neon_cache);
    console_log(cx, &neon_cache_symbol);

    Ok(neon_cache)
}

fn set_ref<'a, V: Value>(
    cx: &mut impl Context<'a>,
    value: Handle<'a, V>,
) -> NeonResult<*mut Value__> {
    let neon_cache = get_cache(cx)?;
    let value_raw = value.to_local();

    get_cache(cx)?
        .get::<JsFunction, _, _>(cx, "add")?
        .call_with(cx)
        .this(neon_cache)
        .arg(value)
        .exec(cx)?;

    Ok(value_raw)
}

fn delete_ref<'a, V: Value>(cx: &mut impl Context<'a>, value: Handle<'a, V>) -> NeonResult<()> {
    let neon_cache = get_cache(cx)?;

    get_cache(cx)?
        .get::<JsFunction, _, _>(cx, "delete")?
        .call_with(cx)
        .this(neon_cache)
        .arg(value)
        .exec(cx)?;

    Ok(())
}

fn console_log<'a, V: Value>(cx: &mut impl Context<'a>, v: &Handle<'a, V>) {
    cx.global::<JsObject>("console")
        .unwrap()
        .get::<JsFunction, _, _>(cx, "log")
        .unwrap()
        .call_with(cx)
        .arg(*v)
        .exec(cx)
        .unwrap();
}
