//! Internals needed by macros. These have to be exported for the macros to work

pub use linkme;

use crate::{context::ModuleContext, handle::Handle, result::NeonResult, types::JsValue};

type Export<'cx> = (&'static str, Handle<'cx, JsValue>);

#[linkme::distributed_slice]
pub static EXPORTS: [for<'cx> fn(&mut ModuleContext<'cx>) -> NeonResult<Export<'cx>>];

#[linkme::distributed_slice]
pub static MAIN: [for<'cx> fn(ModuleContext<'cx>) -> NeonResult<()>];
