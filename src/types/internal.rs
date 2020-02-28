use std::mem;
use std::os::raw::c_void;
use neon_runtime;
use neon_runtime::raw;
use context::{CallbackInfo, FunctionContext};
use context::internal::Env;
use types::error::convert_panics;
use types::{JsObject, Handle, Managed};
use result::JsResult;
use object::class::Callback;
use super::Value;

pub trait ValueInternal: Managed + 'static {
    fn name() -> String;

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool;

    fn downcast<Other: Value>(env: Env, other: Other) -> Option<Self> {
        if Self::is_typeof(env, other) {
            Some(Self::from_raw(other.to_raw()))
        } else {
            None
        }
    }

    fn cast<'a, T: Value, F: FnOnce(raw::Local) -> T>(self, f: F) -> Handle<'a, T> {
        Handle::new_internal(f(self.to_raw()))
    }
}

#[repr(C)]
pub struct FunctionCallback<T: Value>(pub fn(FunctionContext) -> JsResult<T>);

impl<T: Value> Callback<()> for FunctionCallback<T> {
    extern "C" fn invoke(env: Env, info: CallbackInfo) {
        unsafe {
            info.with_cx::<JsObject, _, _>(env, |cx| {
                let data = info.data(env);
                let dynamic_callback: fn(FunctionContext) -> JsResult<T> =
                    mem::transmute(neon_runtime::fun::get_dynamic_callback(env.to_raw(), data));
                if let Ok(value) = convert_panics(|| { dynamic_callback(cx) }) {
                    info.set_return(value);
                }
            })
        }
    }

    fn as_ptr(self) -> *mut c_void {
        unsafe { mem::transmute(self.0) }
    }
}
