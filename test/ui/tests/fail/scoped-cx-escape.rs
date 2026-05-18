//! Soundness regression test: a `ScopedCx<'a, 'b>` must not itself be
//! returnable from the closure passed to `Context::execute_scoped` /
//! `Context::compute_scoped`. Because the closure's return type is fixed
//! outside the HRTB binder on `'b`, there is no way to coerce
//! `ScopedCx<'a, 'b>` (which is parameterized by `'b`) into a `'b`-free
//! return type.

use neon::{
    context::{Context, FunctionContext, ScopedCx},
    result::JsResult,
    types::JsUndefined,
};

fn return_scoped_cx(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let _escaped: ScopedCx<'_, '_> = cx.execute_scoped(|inner| inner);

    Ok(cx.undefined())
}

fn main() {}
