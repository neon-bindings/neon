use once_cell::unsync::Lazy;
use once_cell::unsync::OnceCell;
use std::cell::RefCell;
use std::rc::Rc;

use super::Value;
use crate::context::Context;
use crate::object::Object;
use crate::prelude::Handle;
use crate::result::JsResult;
use crate::result::NeonResult;
use crate::types::JsNumber;
use crate::types::JsObject;

static KEY_NEON_CACHE: &str = "__neon_cache";
static KEY_INSTANCE_KEY: &str = "__instance_count";

thread_local! {
  /// Basic unique key generation
  static COUNT: Lazy<RefCell<u32>> = Lazy::new(|| Default::default());
  static CACHE_KEY: OnceCell<u32> = OnceCell::default();
}

/// Reference counted JavaScript value with a static lifetime for use in async closures
pub struct RootGlobal<T> {
    pub(crate) count: Rc<RefCell<u32>>,
    pub(crate) inner_key: Rc<u32>,
    pub(crate) inner: RefCell<Option<T>>,
}

impl<T: Value> RootGlobal<T> {
    pub(crate) fn new<'a>(
        cx: &mut impl Context<'a>,
        value: Handle<'a, T>,
    ) -> NeonResult<RootGlobal<T>> {
        let inner_key = set_ref(cx, value)?;

        Ok(Self {
            count: Rc::new(RefCell::new(1)),
            inner_key: Rc::new(inner_key),
            inner: Default::default(),
        })
    }

    pub fn clone<'a>(&self, cx: impl Context<'a>) -> Result<RootGlobal<T>, ()> {
        todo!();
    }

    pub fn deref<'a>(&self, cx: impl Context<'a>) -> Result<T, ()> {
        todo!();
    }

    pub fn drop<'a>(&self, cx: impl Context<'a>) -> Result<(), ()> {
        todo!();
    }
}

/*
  globalThis = {
    __napi_cache: {
      __instance_count: number,
      [key: number]: Record<number, any>
    }
  }

  Note: Is there a way to store this privately in the module scope?
*/
fn get_cache<'a>(cx: &mut impl Context<'a>) -> JsResult<'a, JsObject> {
    let global_this = cx.global_object();

    let neon_cache = {
        let neon_cache = global_this.get_opt::<JsObject, _, _>(cx, KEY_NEON_CACHE)?;
        if let Some(neon_cache) = neon_cache {
            neon_cache
        } else {
            let neon_cache = cx.empty_object();
            let initial_count = cx.number(0);
            neon_cache.set(cx, KEY_INSTANCE_KEY, initial_count)?;
            global_this.set(cx, KEY_NEON_CACHE, neon_cache)?;
            global_this.get::<JsObject, _, _>(cx, KEY_NEON_CACHE)?
        }
    };

    let instance_count = CACHE_KEY.with(|key| {
        key.get_or_try_init(|| -> NeonResult<u32> {
            let instance_count = global_this.get::<JsNumber, _, _>(cx, KEY_NEON_CACHE)?;
            let instance_count = instance_count.value(cx) as u32;
            let instance_count = instance_count + 1;
            let instance_count_js = cx.number(instance_count);
            global_this.set::<_, _, JsNumber>(cx, KEY_NEON_CACHE, instance_count_js)?;
            Ok(instance_count)
        })
        .cloned()
    })?;

    neon_cache.get(cx, instance_count)
}

fn set_ref<'a, V: Value>(cx: &mut impl Context<'a>, value: Handle<V>) -> NeonResult<u32> {
    let neon_cache = get_cache(cx)?;

    let key_raw = COUNT.with(|c| {
        let mut c = c.borrow_mut();
        let current = c.clone();
        *c += 1;
        current
    });

    let key = cx.number(key_raw);
    neon_cache.set(cx, key, value)?;
    Ok(key_raw)
}

fn get_ref<'a, T: Value>(cx: &mut impl Context<'a>, key: &u32) -> JsResult<'a, T> {
    let neon_cache = get_cache(cx)?;
    let key = cx.number(key.clone());
    neon_cache.get(cx, key)
}

fn remove_ref<'a>(cx: &mut impl Context<'a>, key: u32) -> NeonResult<()> {
    let neon_cache = get_cache(cx)?;
    let key = cx.number(key);
    let value = cx.undefined();
    neon_cache.set(cx, key, value)?;
    Ok(())
}
