//! Soundness regression test: `Context::compute_scoped` must not allow
//! handles created in the inner scope to escape into the outer scope
//! through closure-captured state. The only sanctioned escape route is
//! the return value, which is rooted in the parent scope via
//! `EscapableHandleScope::escape`.

use neon::{
    context::{Context, FunctionContext},
    handle::Handle,
    result::JsResult,
    types::{JsNumber, JsUndefined},
};

fn leak(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut leaked: Option<Handle<JsNumber>> = None;

    let _: Handle<JsNumber> = cx.compute_scoped(|mut inner_cx| {
        let v = inner_cx.number(42);
        leaked = Some(v);
        Ok(inner_cx.number(0))
    })?;

    Ok(cx.undefined())
}

fn main() {}
