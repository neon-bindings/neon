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
pub static TYPE_METADATA: [ExportMeta];

// ——— TypeScript probes ———
//
// The macros resolve each argument and return type through one of three
// autoref-specialization probes, chosen by the conversion used at that
// position: JSON (`TsJsonProbe`, which lives in `neon-typescript` so adapters
// can hang a rung on it), bare arguments (`TsFromJsProbe`), or bare returns
// (`TsIntoJsProbe`). See `neon::typescript` for details.
pub use crate::typescript::{
    TsFromJs, TsFromJsFallback, TsFromJsProbe, TsIntoJs, TsIntoJsFallback, TsIntoJsProbe,
};
pub use neon_typescript::{TsJsonFallback, TsJsonProbe};

// ——— TypeScript metadata ———
//
// Emitted by the `#[neon::export]` / class macros into `TYPE_METADATA` and read
// by `neon::typescript::generate()`. These types are an implementation detail
// shared by the macros and the generator, not part of Neon's public API, so
// their shape can change in any release.

/// Metadata for a single function parameter.
pub struct ParamMeta {
    /// The parameter name (used in the TypeScript declaration).
    pub name: &'static str,
    /// Returns the TypeScript type expression for this parameter.
    pub ts_type: fn() -> std::borrow::Cow<'static, str>,
    /// Collects type declarations needed by this parameter.
    pub ts_collect: fn(&mut std::collections::BTreeMap<String, String>),
}

/// Metadata for an exported function.
pub struct FunctionMeta {
    /// The JavaScript export name.
    pub name: &'static str,
    /// Parameter metadata.
    pub params: &'static [ParamMeta],
    /// Returns the TypeScript type expression for the return type.
    pub ret_type: fn() -> std::borrow::Cow<'static, str>,
    /// Collects type declarations needed by the return type.
    pub ret_collect: fn(&mut std::collections::BTreeMap<String, String>),
    /// Whether the function returns a Promise.
    pub is_async: bool,
}

/// Metadata for a class method.
pub struct MethodMeta {
    /// The JavaScript method name.
    pub name: &'static str,
    /// Parameter metadata.
    pub params: &'static [ParamMeta],
    /// Returns the TypeScript type expression for the return type.
    pub ret_type: fn() -> std::borrow::Cow<'static, str>,
    /// Collects type declarations needed by the return type.
    pub ret_collect: fn(&mut std::collections::BTreeMap<String, String>),
    /// Whether the method returns a Promise.
    pub is_async: bool,
}

/// Metadata for a class constructor.
pub struct ConstructorMeta {
    /// Constructor parameter metadata.
    pub params: &'static [ParamMeta],
}

/// Metadata for a static class property.
pub struct PropertyMeta {
    /// The JavaScript property name.
    pub name: &'static str,
    /// Returns the TypeScript type expression for this property.
    pub ts_type: fn() -> std::borrow::Cow<'static, str>,
    /// Collects type declarations needed by this property.
    pub ts_collect: fn(&mut std::collections::BTreeMap<String, String>),
}

/// Metadata for an exported class.
pub struct ClassMeta {
    /// The JavaScript class name.
    pub name: &'static str,
    /// Constructor metadata (None if no public constructor).
    pub constructor: Option<ConstructorMeta>,
    /// Instance methods.
    pub methods: &'static [MethodMeta],
    /// Static readonly properties.
    pub static_properties: &'static [PropertyMeta],
}

/// Metadata for an exported item.
pub enum ExportMeta {
    /// A function export.
    Function(FunctionMeta),
    /// A class export.
    Class(ClassMeta),
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
