//! Exposes JavaScript's reflection API to Rust.

use context::Context;
use handle::{Handle, Managed};
use result::JsResult;
use types::{build, JsString, JsValue};

#[cfg(feature = "napi-1")]
pub fn eval<'a, 'b, C: Context<'a>>(
    cx: &mut C,
    script: Handle<'b, JsString>,
) -> JsResult<'a, JsValue> {
    let env = cx.env().to_raw();
    build(cx.env(), |out| unsafe {
        neon_runtime::string::run_script(out, env, script.to_raw())
    })
}
