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

// This creates a rooted object and stores javascript
// values on it as a way to grant any JavaScript value
// a static lifetime

thread_local! {
    static NEON_CACHE: OnceCell<Root<JsObject>> = OnceCell::default();
}

/// Reference counted JavaScript value with a static lifetime for use in async closures
pub struct StaticHandle<T> {
    pub(crate) count: Rc<RefCell<u32>>,
    pub(crate) inner: Rc<Root<JsSymbol>>,
    _p: PhantomData<T>,
}

impl<T: Value> StaticHandle<T> {
    pub(crate) fn new<'a>(
        cx: &mut impl Context<'a>,
        value: Handle<'a, T>,
    ) -> NeonResult<StaticHandle<T>> {
        Ok(Self {
            count: Rc::new(RefCell::new(1)),
            inner: Rc::new(set_ref(cx, value)?),
            _p: Default::default(),
        })
    }

    pub fn clone(&self) -> StaticHandle<T> {
        let mut count = self.count.borrow_mut();
        *count += 1;
        drop(count);

        Self {
            count: self.count.clone(),
            inner: self.inner.clone(),
            _p: self._p.clone(),
        }
    }

    pub fn from_static<'a>(&self, cx: &mut impl Context<'a>) -> JsResult<'a, T> {
        get_ref(cx, &self.inner)
    }

    pub fn drop<'a>(&self, cx: &mut impl Context<'a>) -> NeonResult<()> {
        let mut count = self.count.borrow_mut();
        *count -= 1;

        if *count == 0 {
            delete_ref(cx, &self.inner)?
        }

        Ok(())
    }
}

fn get_cache<'a>(cx: &mut impl Context<'a>) -> JsResult<'a, JsObject> {
    let neon_cache = NEON_CACHE.with({
        |raw_value| {
            raw_value
                .get_or_try_init(|| -> NeonResult<Root<JsObject>> {
                    let set_ctor = cx.global_object().get::<JsFunction, _, _>(cx, "Map")?;
                    let neon_cache = set_ctor.construct(cx, &[])?;
                    Ok(neon_cache.root(cx))
                })
                .and_then(|e| Ok(e.clone(cx)))
        }
    })?;

    Ok(neon_cache.into_inner(cx))
}

fn set_ref<'a, V: Value>(
    cx: &mut impl Context<'a>,
    value: Handle<'a, V>,
) -> NeonResult<Root<JsSymbol>> {
    let neon_cache = get_cache(cx)?;
    let symbol = cx.symbol(format!("{:?}", value.to_local())).root(cx);

    get_cache(cx)?
        .get::<JsFunction, _, _>(cx, "set")?
        .call_with(cx)
        .this(neon_cache)
        .arg(symbol.clone(cx).into_inner(cx))
        .arg(value)
        .exec(cx)?;

    Ok(symbol)
}

fn get_ref<'a, V: Value>(cx: &mut impl Context<'a>, key: &Root<JsSymbol>) -> JsResult<'a, V> {
    let neon_cache = get_cache(cx)?;

    get_cache(cx)?
        .get::<JsFunction, _, _>(cx, "get")?
        .call_with(cx)
        .this(neon_cache)
        .arg(key.clone(cx).into_inner(cx))
        .apply(cx)
}

fn delete_ref<'a>(cx: &mut impl Context<'a>, key: &Root<JsSymbol>) -> NeonResult<()> {
    let neon_cache = get_cache(cx)?;

    get_cache(cx)?
        .get::<JsFunction, _, _>(cx, "delete")?
        .call_with(cx)
        .this(neon_cache)
        .arg(key.clone(cx).into_inner(cx))
        .exec(cx)?;

    Ok(())
}
