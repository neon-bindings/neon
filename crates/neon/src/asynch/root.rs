/*
    This is a basic method of persisting JavaScript values so
    they are given a static lifetime and not collected by GC

    This is needed because not all JsValues can be called with .root()
    and is probably temporary
*/
use std::cell::RefCell;

use crate::context::Context;
use crate::handle::Handle;
use crate::object::Object;
use crate::types::JsObject;
use crate::types::Value;
use once_cell::unsync::Lazy;

thread_local! {
    pub static THREAD_LOCAL_COUNT: Lazy<RefCell<usize>> = Lazy::new(|| RefCell::new(0));
    pub static GLOBAL_KEY: Lazy<String> = Lazy::new(|| {
        let mut lower = [0; std::mem::size_of::<u16>()];
        getrandom::getrandom(&mut lower).expect("Unable to generate number");
        let lower = u16::from_ne_bytes(lower);
        format!("__neon_root_cache_{}", lower)
    });
}

fn ref_count_inc() -> usize {
    THREAD_LOCAL_COUNT.with(|c| {
        let mut c = c.borrow_mut();
        let current = (*c).clone();
        *c += 1;
        current
    })
}

pub fn root<'a>(cx: &mut impl Context<'a>) -> Handle<'a, JsObject> {
    let global = cx.global_object();
    let key = GLOBAL_KEY.with(|k| cx.string(&*k.as_str()));
    match global.get_opt(cx, key).unwrap() {
        Some(obj) => obj,
        None => {
            let init = cx.empty_object();
            global.set(cx, key, init).unwrap();
            global.get_opt(cx, key).unwrap().unwrap()
        }
    }
}

#[derive(Clone, Debug)]
pub struct RootGlobal {
    inner: String,
}

impl RootGlobal {
    pub fn new<'a, V: Value>(cx: &mut impl Context<'a>, value: Handle<V>) -> Self {
        let index = ref_count_inc();
        let key_str = format!("{}", index);
        let key = cx.string(&key_str);
        let cache = root(cx);
        cache.set(cx, key, value).unwrap();
        Self { inner: key_str }
    }

    pub fn into_inner<'a, V: Value>(&self, cx: &mut impl Context<'a>) -> Handle<'a, V> {
        let key = cx.string(&self.inner);
        let cache = root(cx);
        cache.get(cx, key).unwrap()
    }

    pub fn remove<'a>(&self, cx: &mut impl Context<'a>) -> bool {
        let key = cx.string(&self.inner);
        let val = cx.undefined();
        let cache = root(cx);
        cache.set(cx, key, val).unwrap()
    }
}
