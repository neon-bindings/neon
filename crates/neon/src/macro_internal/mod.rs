//! Internals needed by macros. These have to be exported for the macros to work

pub use linkme;
pub use crate::context::internal::initialize_module;

use crate::{context::ModuleContext, handle::Handle, result::NeonResult, types::JsValue};

#[linkme::distributed_slice]
pub static EXPORTS: [for<'cx> fn(&mut ModuleContext<'cx>) -> NeonResult<(&'cx str, Handle<'cx, JsValue>)>];
