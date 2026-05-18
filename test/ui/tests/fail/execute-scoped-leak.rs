//! Soundness regression test: `Context::execute_scoped` must not allow
//! handles created in the inner scope to escape into the outer scope.
//!
//! Before the HRTB fix, the caller could pick `'b = 'a`, allowing the
//! closure to leak a `Handle<'a, _>` allocated inside the temporary
//! `HandleScope` into the outer context via a captured `&mut`, producing
//! a use-after-free once the inner scope was dropped.

use neon::{
    context::{Context, FunctionContext},
    handle::Handle,
    result::JsResult,
    types::{JsNumber, JsUndefined},
};

fn leak(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut leaked: Option<Handle<JsNumber>> = None;

    cx.execute_scoped(|mut inner_cx| {
        leaked = Some(inner_cx.number(42));
    });

    Ok(cx.undefined())
}

fn main() {}
