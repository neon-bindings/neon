use super::{private::ValueInternal, Value};
use crate::context::internal::Env;
use crate::context::Context;
use crate::handle::{internal::TransparentNoCopyWrapper, Handle, Managed};
use neon_runtime;
use neon_runtime::raw;
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

#[derive(Debug)]
pub struct GetBigIntLossyValueResult<T> {
    pub value: T,
    /// Indicates whether the BigInt value was converted losslessly
    pub lossless: bool,
}

impl<T> fmt::Display for GetBigIntLossyValueResult<T>
where
    T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Into<i64> for GetBigIntLossyValueResult<i64> {
    fn into(self) -> i64 {
        self.value
    }
}

impl Into<u64> for GetBigIntLossyValueResult<u64> {
    fn into(self) -> u64 {
        self.value
    }
}

impl JsBigInt {
    pub fn new<'a, C: Context<'a>, T: Into<i64>>(cx: &mut C, x: T) -> Handle<'a, JsBigInt> {
        let env = cx.env().to_raw();
        let value = x.into();
        let local = unsafe { neon_runtime::bigint::new_bigint(env, value) };
        let bigint = Handle::new_internal(JsBigInt(local));
        bigint
    }

    pub fn from_u64<'a, C: Context<'a>, T: Into<u64>>(cx: &mut C, x: T) -> Handle<'a, JsBigInt> {
        let env = cx.env().to_raw();
        let value = x.into();
        let local = unsafe { neon_runtime::bigint::new_bigint_from_u64(env, value) };
        let bigint = Handle::new_internal(JsBigInt(local));
        bigint
    }

    pub fn value_i64<'a, C: Context<'a>>(self, cx: &mut C) -> GetBigIntLossyValueResult<i64> {
        let env = cx.env().to_raw();
        let result = unsafe { neon_runtime::bigint::value_i64(env, self.to_raw()) };
        GetBigIntLossyValueResult {
            value: result.0,
            lossless: result.1,
        }
    }

    pub fn value_u64<'a, C: Context<'a>>(self, cx: &mut C) -> GetBigIntLossyValueResult<u64> {
        let env = cx.env().to_raw();
        let result = unsafe { neon_runtime::bigint::value_u64(env, self.to_raw()) };
        GetBigIntLossyValueResult {
            value: result.0,
            lossless: result.1,
        }
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
