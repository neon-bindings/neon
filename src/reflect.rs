//! Exposes JavaScript's reflection API to Rust.

use crate::{
    context::Context,
    handle::{Handle, Managed},
    result::JsResult,
    types::{build, JsString, JsValue},
};

pub fn eval<'a, 'b, C: Context<'a>>(
    cx: &mut C,
    script: Handle<'b, JsString>,
) -> JsResult<'a, JsValue> {
    let env = cx.env().to_raw();
    build(cx.env(), |out| unsafe {
        crate::sys::string::run_script(out, env, script.to_raw())
    })
}
