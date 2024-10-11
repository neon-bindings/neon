use std::{error, fmt, mem::MaybeUninit};

use crate::{
    context::{internal::Env, Context},
    handle::{internal::TransparentNoCopyWrapper, root::NapiRef, Handle},
    macro_internal::NeonResultTag,
    object::Object,
    result::{JsResult, NeonResult, ResultExt},
    sys::{self, raw},
    types::{private, JsFunction, JsObject, Value},
};

use super::{JsBoolean, JsNumber, JsUndefined};

#[derive(Debug)]
#[repr(transparent)]
/// The type of JavaScript
/// [`Map`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map)
/// objects.
pub struct JsMap(raw::Local);

impl JsMap {
    pub fn new<'cx, C>(cx: &mut C) -> NeonResult<Handle<'cx, Self>>
    where
        C: Context<'cx>,
    {
        let map = cx
            .global::<JsFunction>("Map")?
            .construct_with(cx)
            .apply::<JsObject, _>(cx)?;

        Ok(map.downcast_or_throw(cx)?)
    }

    pub fn size<'a, C: Context<'a>>(&self, cx: &mut C) -> NeonResult<Handle<'a, JsNumber>> {
        Object::get(self, cx, "size")
    }

    // TODO: is the return type important here ?
    // I see three possibilities here:
    // 1. Stick to the JS and return the `undefined` (this is what we do now)
    // 2. Check we get an `undefined` and return `Ok(())`
    // 3. Just discard the return value and return `Ok(())`
    // Solutions 2 & 3 are more user-friendly, but make more assumptions (though it
    // should be fine given `Map` is not expected to be overridden ?).
    pub fn clear<'a, C: Context<'a>>(&self, cx: &mut C) -> NeonResult<Handle<'a, JsUndefined>> {
        Object::call_method_with(self, cx, "clear")?.apply(cx)
    }

    pub fn delete<'a, C: Context<'a>, K: Value>(
        &self,
        cx: &mut C,
        key: Handle<'a, K>,
    ) -> NeonResult<bool> {
        Object::call_method_with(self, cx, "delete")?
            .arg(key)
            .apply::<JsBoolean, _>(cx)
            .map(|v| v.value(cx))
    }

    pub fn entries<'a, C: Context<'a>>(&self, cx: &mut C) -> NeonResult<Handle<'a, JsObject>> {
        Object::call_method_with(self, cx, "entries")?.apply(cx)
    }

    pub fn for_each<'a, C: Context<'a>, F: Value>(
        &self,
        cx: &mut C,
        cb: Handle<'a, F>,
    ) -> NeonResult<Handle<'a, JsUndefined>> {
        Object::call_method_with(self, cx, "forEach")?
            .arg(cb)
            .apply(cx)
    }

    pub fn get<'a, C: Context<'a>, K: Value, R: Value>(
        &self,
        cx: &mut C,
        key: Handle<'a, K>,
    ) -> NeonResult<Handle<'a, R>> {
        Object::call_method_with(self, cx, "get")?
            .arg(key)
            .apply(cx)
    }

    pub fn has<'a, C: Context<'a>, K: Value>(
        &self,
        cx: &mut C,
        key: Handle<'a, K>,
    ) -> NeonResult<bool> {
        Object::call_method_with(self, cx, "has")?
            .arg(key)
            .apply::<JsBoolean, _>(cx)
            .map(|v| v.value(cx))
    }

    pub fn keys<'a, C: Context<'a>, R: Value>(&self, cx: &mut C) -> NeonResult<Handle<'a, R>> {
        Object::call_method_with(self, cx, "keys")?.apply(cx)
    }

    pub fn set<'a, C: Context<'a>, K: Value, V: Value>(
        &self,
        cx: &mut C,
        key: Handle<'a, K>,
        value: Handle<'a, V>,
    ) -> NeonResult<Handle<'a, JsMap>> {
        Object::call_method_with(self, cx, "set")?
            .arg(key)
            .arg(value)
            .apply(cx)
    }

    pub fn values<'a, C: Context<'a>, R: Value>(&self, cx: &mut C) -> NeonResult<Handle<'a, R>> {
        Object::call_method_with(self, cx, "values")?.apply(cx)
    }

    pub fn group_by<'a, C: Context<'a>, A: Value, B: Value, R: Value>(
        cx: &mut C,
        elements: Handle<'a, A>,
        cb: Handle<'a, B>,
    ) -> NeonResult<Handle<'a, R>> {
        // TODO: This is broken and leads to a `failed to downcast any to object` error
        // when trying to downcast `Map.groupBy` into a `JsFunction`...
        cx.global::<JsObject>("Map")?
            .call_method_with(cx, "groupBy")?
            .arg(elements)
            .arg(cb)
            .apply(cx)
    }

    // TODO: should we implementd those as well ?
    // Map[Symbol.species]
    // Map.prototype[Symbol.iterator]()
}

unsafe impl TransparentNoCopyWrapper for JsMap {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl private::ValueInternal for JsMap {
    fn name() -> &'static str {
        "Map"
    }

    fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool {
        unsafe { sys::tag::is_map(env.to_raw(), other.to_local()) }
    }

    fn to_local(&self) -> raw::Local {
        self.0
    }

    unsafe fn from_local(_env: Env, h: raw::Local) -> Self {
        Self(h)
    }
}

impl Value for JsMap {}

impl Object for JsMap {}
