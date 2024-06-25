use once_cell::unsync::OnceCell;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::Root;
use super::Value;
use crate::context::Context;
use crate::object::Object;
use crate::prelude::Handle;
use crate::result::JsResult;
use crate::result::NeonResult;
use crate::types::JsFunction;
use crate::types::JsObject;
use crate::types::JsSymbol;

static KEY_NEON_CACHE: &str = "__neon_cache";

thread_local! {
    // Symbol("__neon_cache")
    static CACHE_SYMBOL: OnceCell<Root<JsSymbol>> = OnceCell::default();
}

/// Reference counted JavaScript value with a static lifetime for use in async closures
pub struct RootGlobal<T> {
    pub(crate) count: Rc<RefCell<u32>>,
    pub(crate) inner: Rc<String>,
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

    pub fn clone<'a>(&self) -> RootGlobal<T> {
        let mut count = self.count.borrow_mut();
        *count += 1;
        drop(count);

        Self {
            count: self.count.clone(),
            inner: self.inner.clone(),
            _p: self._p.clone(),
        }
    }

    pub fn into_inner<'a>(&self, cx: &mut impl Context<'a>) -> JsResult<'a, T> {
        get_ref(cx, &*self.inner)
    }

    pub fn drop<'a>(&self, cx: &mut impl Context<'a>) -> NeonResult<()> {
        let mut count = self.count.borrow_mut();
        *count -= 1;

        if *count == 0 {
            delete_ref(cx, &*self.inner)?
        }

        Ok(())
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
                .get_or_try_init(|| -> NeonResult<Root<JsSymbol>> {
                    let set_ctor = global_this.get::<JsFunction, _, _>(cx, "Map")?;
                    let neon_cache = set_ctor.construct(cx, &[])?;

                    let symbol = cx.symbol(KEY_NEON_CACHE);
                    let symbol = symbol.root(cx);

                    {
                        let symbol = symbol.clone(cx);
                        let symbol = symbol.into_inner(cx);
                        global_this.set(cx, symbol, neon_cache)?;
                    }

                    Ok(symbol)
                })
                .and_then(|e| Ok(e.clone(cx)))
        }
    })?;

    let neon_cache_symbol = neon_cache_symbol.into_inner(cx);

    let Some(neon_cache) = global_this.get_opt::<JsObject, _, _>(cx, neon_cache_symbol)? else {
        return Err(cx.throw_error("Unable to find cache")?);
    };

    Ok(neon_cache)
}

fn set_ref<'a, V: Value>(cx: &mut impl Context<'a>, value: Handle<'a, V>) -> NeonResult<String> {
    let neon_cache = get_cache(cx)?;
    let key = format!("{:?}", value.to_local());

    get_cache(cx)?
        .get::<JsFunction, _, _>(cx, "set")?
        .call_with(cx)
        .this(neon_cache)
        .arg(cx.string(&key))
        .arg(value)
        .exec(cx)?;

    Ok(key)
}

fn get_ref<'a, V: Value>(cx: &mut impl Context<'a>, key: &str) -> JsResult<'a, V> {
    let neon_cache = get_cache(cx)?;

    get_cache(cx)?
        .get::<JsFunction, _, _>(cx, "get")?
        .call_with(cx)
        .this(neon_cache)
        .arg(cx.string(&key))
        .apply(cx)
}

fn delete_ref<'a>(cx: &mut impl Context<'a>, key: &str) -> NeonResult<()> {
    let neon_cache = get_cache(cx)?;

    get_cache(cx)?
        .get::<JsFunction, _, _>(cx, "delete")?
        .call_with(cx)
        .this(neon_cache)
        .arg(cx.string(&key))
        .exec(cx)?;

    Ok(())
}
