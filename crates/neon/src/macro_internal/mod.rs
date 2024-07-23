//! Internals needed by macros. These have to be exported for the macros to work

pub use linkme;

use crate::{
    context::{Context, ModuleContext},
    handle::Handle,
    result::{JsResult, NeonResult},
    types::{extract::TryIntoJs, JsValue},
};

#[cfg(all(feature = "napi-6", feature = "futures"))]
pub use self::futures::*;

#[cfg(all(feature = "napi-6", feature = "futures"))]
mod futures;

type Export<'cx> = (&'static str, Handle<'cx, JsValue>);

#[linkme::distributed_slice]
pub static EXPORTS: [for<'cx> fn(&mut ModuleContext<'cx>) -> NeonResult<Export<'cx>>];

#[linkme::distributed_slice]
pub static MAIN: [for<'cx> fn(ModuleContext<'cx>) -> NeonResult<()>];

pub trait NeonExportReturnValue<'cx> {
    fn try_neon_export_return<C>(self, cx: &mut C) -> JsResult<'cx, JsValue>
    where
        C: Context<'cx>;
}

impl<'cx, T> NeonExportReturnValue<'cx> for T
where
    T: TryIntoJs<'cx>,
{
    fn try_neon_export_return<C>(self, cx: &mut C) -> JsResult<'cx, JsValue>
    where
        C: Context<'cx>,
    {
        self.try_into_js(cx).map(|v| v.upcast())
    }
}

#[cfg(feature = "serde")]
pub trait NeonExportReturnJson<'cx> {
    fn try_neon_export_return<C>(self, cx: &mut C) -> JsResult<'cx, JsValue>
    where
        C: Context<'cx>;
}

#[cfg(feature = "serde")]
impl<'cx, T, E> NeonExportReturnJson<'cx> for Result<T, E>
where
    T: serde::Serialize,
    E: TryIntoJs<'cx>,
{
    fn try_neon_export_return<C>(self, cx: &mut C) -> JsResult<'cx, JsValue>
    where
        C: Context<'cx>,
    {
        self.map(crate::types::extract::Json).try_into_js(cx)
    }
}

#[cfg(feature = "serde")]
impl<'cx, T> NeonExportReturnJson<'cx> for &T
where
    T: serde::Serialize,
{
    fn try_neon_export_return<C>(self, cx: &mut C) -> JsResult<'cx, JsValue>
    where
        C: Context<'cx>,
    {
        crate::types::extract::Json(self).try_into_js(cx)
    }
}
