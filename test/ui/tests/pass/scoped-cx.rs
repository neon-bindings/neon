//! Pass test for `Context::execute_scoped` and `Context::compute_scoped`:
//! the legitimate patterns (using outer-scope handles inside the closure,
//! and re-escaping an outer-scope handle from `compute_scoped`) must still
//! compile under the higher-ranked `ScopedCx<'a, 'b>` signatures.

use neon::{
    context::{Context, FunctionContext},
    handle::Handle,
    object::Object,
    result::JsResult,
    types::{JsNumber, JsObject, JsValue},
};

/// `execute_scoped` closure may freely use captured outer-scope handles
/// alongside inner-scope handles, as long as nothing inner-scope escapes.
fn use_outer_in_execute_scoped(mut cx: FunctionContext) -> JsResult<JsValue> {
    let outer: Handle<JsObject> = cx.argument(0)?;

    for i in 0..3u32 {
        cx.execute_scoped(|mut inner_cx| -> Result<(), neon::result::Throw> {
            let n = inner_cx.number(i);
            outer.set(&mut inner_cx, i, n)?;
            Ok(())
        })?;
    }

    Ok(outer.upcast())
}

/// `compute_scoped` can pass an outer-scope handle through unchanged: the
/// implicit `'a: 'b` bound carried by `ScopedCx<'a, 'b>` lets `Handle<'a, V>`
/// be coerced to `Handle<'b, V>`, and the escape mechanism then re-roots it
/// in the parent scope.
fn pass_through_compute_scoped(mut cx: FunctionContext) -> JsResult<JsValue> {
    let value: Handle<JsValue> = cx.argument(0)?;

    cx.compute_scoped(move |_| Ok(value))
}

/// `compute_scoped` can also allocate a fresh handle inside the scope and
/// return it; the value is escaped via `EscapableHandleScope` into the
/// parent scope.
fn allocate_in_compute_scoped(mut cx: FunctionContext) -> JsResult<JsNumber> {
    cx.compute_scoped(|mut inner_cx| Ok(inner_cx.number(42)))
}

fn main() {}
