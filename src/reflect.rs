//! Exposes JavaScript's reflection API to Rust.

use crate::context::Context;
use crate::handle::{Handle, Managed};
use crate::result::JsResult;
use crate::types::{build, JsString, JsValue};

pub fn eval<'a, 'b, C: Context<'a>>(
    cx: &mut C,
    script: Handle<'b, JsString>,
) -> JsResult<'a, JsValue> {
    let env = cx.env().to_raw();
    build(cx.env(), |out| unsafe {
        neon_runtime::string::run_script(out, env, script.to_raw())
    })
}
