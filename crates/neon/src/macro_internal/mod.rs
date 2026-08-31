//! Internals needed by macros. These have to be exported for the macros to work

use std::marker::PhantomData;

pub use linkme;

use crate::{
    context::{Context, Cx, ModuleContext},
    handle::Handle,
    result::{JsResult, NeonResult},
    types::{extract::TryIntoJs, JsValue},
};

#[cfg(feature = "serde")]
use crate::types::extract::Json;

#[cfg(all(feature = "napi-6", feature = "futures"))]
pub use self::futures::*;

pub mod object {
    pub use crate::object::wrap::{unwrap, wrap};
}

#[cfg(all(feature = "napi-6", feature = "futures"))]
mod futures;

type Export<'cx> = (&'static str, Handle<'cx, JsValue>);

#[linkme::distributed_slice]
pub static EXPORTS: [for<'cx> fn(&mut ModuleContext<'cx>) -> NeonResult<Export<'cx>>];

#[linkme::distributed_slice]
pub static MAIN: [for<'cx> fn(ModuleContext<'cx>) -> NeonResult<()>];

#[linkme::distributed_slice]
pub static TYPE_METADATA: [crate::typescript::ExportMeta];

// ——— TypeScript autoref specialization probe ———
//
// This enables macro-generated metadata to gracefully handle types that don't
// implement `TypeScript`. Method resolution prefers the inherent impl (which
// requires `T: TypeScript`) over the trait impl (which works for all `T` via
// autoref). Types with impls get real type info; types without get `"any"`.
//
// Usage in macro-generated code:
//   let __probe = TsProbe::<SomeType>(PhantomData);
//   (&__probe).ts_type_of()

/// Compile-time probe for whether a type implements `TypeScript`.
pub struct TsProbe<T>(pub PhantomData<T>);

// Higher priority: inherent method, available only when T: TypeScript
impl<T: crate::typescript::TypeScript> TsProbe<T> {
    pub fn ts_type_of(&self) -> std::borrow::Cow<'static, str> {
        T::ts_type()
    }

    pub fn ts_collect_of(&self, decls: &mut std::collections::BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

/// Lower-priority fallback for types that don't implement `TypeScript`.
/// Reached via autoref (`(&probe).method()` resolves to `&TsProbe<T>` first,
/// then falls through to `&&TsProbe<T>` which matches this trait impl).
pub trait TsFallback {
    fn ts_type_of(&self) -> std::borrow::Cow<'static, str>;
    fn ts_collect_of(&self, decls: &mut std::collections::BTreeMap<String, String>);
}

impl<T> TsFallback for &TsProbe<T> {
    fn ts_type_of(&self) -> std::borrow::Cow<'static, str> {
        "any".into()
    }

    fn ts_collect_of(&self, _: &mut std::collections::BTreeMap<String, String>) {}
}

// Wrapper for the value type and return type tags
pub struct NeonMarker<Tag, Return>(PhantomData<Tag>, PhantomData<Return>);

// Markers to determine the type of a value
#[cfg(feature = "serde")]
pub struct NeonJsonTag;
pub struct NeonValueTag;
pub struct NeonResultTag;

pub trait ToNeonMarker {
    type Return;

    fn to_neon_marker<Tag>(&self) -> NeonMarker<Tag, Self::Return>;
}

// Specialized implementation for `Result`
impl<T, E> ToNeonMarker for Result<T, E> {
    type Return = NeonResultTag;

    fn to_neon_marker<Tag>(&self) -> NeonMarker<Tag, Self::Return> {
        NeonMarker(PhantomData, PhantomData)
    }
}

// Default implementation that takes lower precedence due to autoref
impl<T> ToNeonMarker for &T {
    type Return = NeonValueTag;

    fn to_neon_marker<Tag>(&self) -> NeonMarker<Tag, Self::Return> {
        NeonMarker(PhantomData, PhantomData)
    }
}

impl<Return> NeonMarker<NeonValueTag, Return> {
    pub fn neon_into_js<'cx, T>(self, cx: &mut Cx<'cx>, v: T) -> JsResult<'cx, JsValue>
    where
        T: TryIntoJs<'cx>,
    {
        v.try_into_js(cx).map(|v| v.upcast())
    }
}

#[cfg(feature = "serde")]
impl NeonMarker<NeonJsonTag, NeonValueTag> {
    pub fn neon_into_js<'cx, T>(self, cx: &mut Cx<'cx>, v: T) -> JsResult<'cx, JsValue>
    where
        Json<T>: TryIntoJs<'cx>,
    {
        Json(v).try_into_js(cx).map(|v| v.upcast())
    }
}

#[cfg(feature = "serde")]
impl NeonMarker<NeonJsonTag, NeonResultTag> {
    pub fn neon_into_js<'cx, T, E>(
        self,
        cx: &mut Cx<'cx>,
        res: Result<T, E>,
    ) -> JsResult<'cx, JsValue>
    where
        Result<Json<T>, E>: TryIntoJs<'cx>,
    {
        res.map(Json).try_into_js(cx).map(|v| v.upcast())
    }
}

impl<Tag> NeonMarker<Tag, NeonValueTag> {
    pub fn into_neon_result<T>(self, _cx: &mut Cx, v: T) -> NeonResult<T> {
        Ok(v)
    }
}

impl<Tag> NeonMarker<Tag, NeonResultTag> {
    pub fn into_neon_result<'cx, T, E>(self, cx: &mut Cx<'cx>, res: Result<T, E>) -> NeonResult<T>
    where
        E: TryIntoJs<'cx>,
    {
        match res {
            Ok(v) => Ok(v),
            Err(err) => err.try_into_js(cx).and_then(|err| cx.throw(err)),
        }
    }
}

#[cfg(feature = "napi-6")]
pub use crate::object::class::new_class_metadata;

#[cfg(feature = "napi-6")]
pub use crate::types_impl::extract::private::Sealed;

#[cfg(feature = "napi-6")]
pub use crate::object::wrap::WrapError;

#[cfg(feature = "napi-6")]
pub use crate::object::class::ClassInternal;

#[cfg(feature = "napi-6")]
pub use crate::object::class::{ClassMetadata, RootClassMetadata};

#[cfg(feature = "napi-6")]
pub fn internal_constructor<'cx, T: crate::object::Class>(
    cx: &mut Cx<'cx>,
) -> NeonResult<Handle<'cx, crate::types::JsFunction>> {
    let class_instance = T::local(cx)?;
    Ok(class_instance.internal_constructor())
}
