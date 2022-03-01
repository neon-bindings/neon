use super::{private::ValueInternal, Value};
use crate::context::internal::Env;
use crate::context::Context;
use crate::handle::{internal::TransparentNoCopyWrapper, Handle, Managed};
use crate::object::Object;
use crate::result::{JsResult, JsResultExt};
use neon_runtime;
use neon_runtime::raw;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
pub struct JsBigInt(raw::Local);

impl Value for JsBigInt {}

unsafe impl TransparentNoCopyWrapper for JsBigInt {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl Managed for JsBigInt {
    fn to_raw(&self) -> raw::Local {
        self.0
    }

    fn from_raw(_: Env, h: raw::Local) -> Self {
        JsBigInt(h)
    }
}

impl JsBigInt {
    pub fn new<'a, C: Context<'a>, T: Into<i64>>(cx: &mut C, x: T) -> Handle<'a, JsBigInt> {
        // JsBigInt::new_internal(cx.env(), x.into())
        let env = cx.env().to_raw();
        let value = x.into();
        let local = unsafe { neon_runtime::bigint::new_bigint(env, value) };
        let bigint = Handle::new_internal(JsBigInt(local));
        bigint
    }

    /*
    pub(crate) fn new_internal<'a>(env: Env, v: i64) -> Handle<'a, JsBigInt> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::primitive::bigint(&mut local, env.to_raw(), v);
            Handle::new_internal(JsBigInt(local))
        }
    }
    */

    pub fn value<'a, C: Context<'a>>(self, cx: &mut C) -> i64 {
        let env = cx.env().to_raw();
        unsafe { neon_runtime::bigint::value_i64(env, self.to_raw()) }
    }

}

impl ValueInternal for JsBigInt {
    fn name() -> String {
        "bigint".to_string()
    }

    fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_bigint(env.to_raw(), other.to_raw()) }
    }
}
