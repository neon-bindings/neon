//! Exposes JavaScript's reflection API to Rust.

use crate::{
    context::Context,
    handle::Handle,
    result::{JsResult, Throw},
    sys::raw,
    types::{build, private::ValueInternal, JsString, JsValue},
};

pub fn eval<'a, 'b, C: Context<'a>>(
    cx: &mut C,
    script: Handle<'b, JsString>,
) -> JsResult<'a, JsValue> {
    let env = cx.env();
    build(env, move || unsafe {
        let mut out: raw::Local = std::ptr::null_mut();
        crate::sys::string::run_script(&mut out, env.to_raw(), script.to_local())
            .then_some(out)
            .ok_or(Throw::new())
    })
}
