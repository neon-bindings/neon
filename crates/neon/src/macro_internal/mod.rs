//! Internals needed by macros. These have to be exported for the macros to work

pub use linkme;

use crate::{
    context::{Cx, ModuleContext},
    handle::Handle,
    result::{JsResult, NeonResult},
    types::{extract::TryIntoJs, JsValue},
};

type Export<'cx> = (&'static str, Handle<'cx, JsValue>);

#[linkme::distributed_slice]
pub static EXPORTS: [for<'cx> fn(&mut ModuleContext<'cx>) -> NeonResult<Export<'cx>>];

#[linkme::distributed_slice]
pub static MAIN: [for<'cx> fn(ModuleContext<'cx>) -> NeonResult<()>];

// Provides an identically named method to `NeonExportReturnJson` for easy swapping in macros
pub trait NeonExportReturnValue<'cx> {
    fn try_neon_export_return(self, cx: &mut Cx<'cx>) -> JsResult<'cx, JsValue>;
}

impl<'cx, T> NeonExportReturnValue<'cx> for T
where
    T: TryIntoJs<'cx>,
{
    fn try_neon_export_return(self, cx: &mut Cx<'cx>) -> JsResult<'cx, JsValue> {
        self.try_into_js(cx).map(|v| v.upcast())
    }
}

#[cfg(feature = "serde")]
// Trait used for specializing `Json` wrapping of `T` or `Result<T, _>` in macros
// Leverages the [autoref specialization](https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md) technique
pub trait NeonExportReturnJson<'cx> {
    fn try_neon_export_return(self, cx: &mut Cx<'cx>) -> JsResult<'cx, JsValue>;
}

#[cfg(feature = "serde")]
// More specific behavior wraps `Result::Ok` in `Json`
impl<'cx, T, E> NeonExportReturnJson<'cx> for Result<T, E>
where
    T: serde::Serialize,
    E: TryIntoJs<'cx>,
{
    fn try_neon_export_return(self, cx: &mut Cx<'cx>) -> JsResult<'cx, JsValue> {
        self.map(crate::types::extract::Json).try_into_js(cx)
    }
}

#[cfg(feature = "serde")]
// Due to autoref behavior, this is less specific than the other implementation
impl<'cx, T> NeonExportReturnJson<'cx> for &T
where
    T: serde::Serialize,
{
    fn try_neon_export_return(self, cx: &mut Cx<'cx>) -> JsResult<'cx, JsValue> {
        crate::types::extract::Json(self).try_into_js(cx)
    }
}
