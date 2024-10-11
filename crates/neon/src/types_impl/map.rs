use crate::{
    context::{internal::Env, Context, Cx}, handle::{internal::TransparentNoCopyWrapper, Handle, Root}, object::Object, result::{JsResult, NeonResult}, sys::raw, thread::LocalKey, types::{private, JsFunction, JsObject, Value}
};

use super::extract::{TryFromJs, TryIntoJs};

#[derive(Debug)]
#[repr(transparent)]
/// The type of JavaScript
/// [`Map`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map)
/// objects.
pub struct JsMap(raw::Local);

impl JsMap {
    pub fn new<'cx>(cx: &mut Cx<'cx>) -> NeonResult<Handle<'cx, Self>> {
        let map = cx
            .global::<JsFunction>("Map")?
            .construct_with(cx)
            .apply::<JsObject, _>(cx)?;

        Ok(map.downcast_or_throw(cx)?)
    }

    pub fn size(&self, cx: &mut Cx) -> NeonResult<f64> {
        self.prop(cx, "size").get()
    }

    pub fn clear(&self, cx: &mut Cx) -> NeonResult<()> {
        self.method(cx, "clear")?.call()
    }

    pub fn delete<'cx, K>(
        &self,
        cx: &mut Cx<'cx>,
        key: K,
    ) -> NeonResult<bool>
    where K: TryIntoJs<'cx> {
        self.method(cx, "delete")?.arg(key)?.call()
    }

    pub fn entries<'cx, R>(&self, cx: &mut Cx<'cx>) -> NeonResult<R>
    where R: TryFromJs<'cx>
    {
        self.method(cx, "entries")?.call()
    }

    pub fn for_each<'cx, F, R>(
        &self,
        cx: &mut Cx<'cx>,
        cb: F,
    ) -> NeonResult<R>
    where F: TryIntoJs<'cx>, R: TryFromJs<'cx>
    {
        self.method(cx, "forEach")?.arg(cb)?.call()
    }

    pub fn get<'cx, K, R>(
        &self,
        cx: &mut Cx<'cx>,
        key: K,
    ) -> NeonResult<R>
    where
        K: TryIntoJs<'cx>,
        R: TryFromJs<'cx>
    {
        self.method(cx, "get")?.arg(key)?.call()
    }

    pub fn has<'cx, K>(
        &self,
        cx: &mut Cx<'cx>,
        key: K,
    ) -> NeonResult<bool>
    where
        K: TryIntoJs<'cx>,
    {
        self.method(cx, "has")?.arg(key)?.call()
    }

    pub fn keys<'cx, R>(&self, cx: &mut Cx<'cx>) -> NeonResult<R>
    where
        R: TryFromJs<'cx>
    {
        self.method(cx, "keys")?.call()
    }

    pub fn set<'cx, K, V>(
        &self,
        cx: &mut Cx<'cx>,
        key: K,
        value: V,
    ) -> NeonResult<Handle<'cx, JsMap>>
    where
        K: TryIntoJs<'cx>,
        V: TryIntoJs<'cx>
    {
        self.method(cx, "set")?.arg(key)?.arg(value)?.call()
    }

    pub fn values<'cx, R>(&self, cx: &mut Cx<'cx>) -> NeonResult<R>
    where
        R: TryFromJs<'cx>
    {
        self.method(cx, "values")?.call()
    }

    pub fn group_by<'cx, A, B, R>(
        cx: &mut Cx<'cx>,
        elements: A,
        cb: B,
    ) -> NeonResult<R>
    where
        A: TryIntoJs<'cx>,
        B: TryIntoJs<'cx>,
        R: TryFromJs<'cx>
    {
        // TODO: This is broken and leads to a `failed to downcast any to object` error
        // when trying to downcast `Map.groupBy` into a `JsFunction`...
        cx.global::<JsObject>("Map")?
            .method(cx, "groupBy")?
            .arg(elements)?
            .arg(cb)?
            .call()
    }
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
        Cx::with_context(env, |mut cx| {
            let ctor = map_constructor(&mut cx).unwrap();
            other.instance_of(&mut cx, &*ctor)
        })
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

fn global_map_constructor<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction> {
    cx.global::<JsFunction>("Map")
}

fn map_constructor<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction> {
    static MAP_CONSTRUCTOR: LocalKey<Root<JsFunction>> = LocalKey::new();

    MAP_CONSTRUCTOR
         .get_or_try_init(cx, |cx| global_map_constructor(cx).map(|f| f.root(cx)))
         .map(|f| f.to_inner(cx))
}
