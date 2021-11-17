use super::Value;
use crate::context::internal::Env;
#[cfg(feature = "legacy-runtime")]
use crate::context::CallbackInfo;
#[cfg(feature = "legacy-runtime")]
use crate::context::FunctionContext;
#[cfg(feature = "legacy-runtime")]
use crate::result::JsResult;
use crate::types::{Handle, Managed};
use neon_runtime;
#[cfg(feature = "legacy-runtime")]
use neon_runtime::call::CCallback;
use neon_runtime::raw;
#[cfg(feature = "legacy-runtime")]
use std::os::raw::c_void;

pub trait ValueInternal: Managed + 'static {
    fn name() -> String;

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool;

    fn downcast<Other: Value>(env: Env, other: Other) -> Option<Self> {
        if Self::is_typeof(env, other) {
            Some(Self::from_raw(env, other.to_raw()))
        } else {
            None
        }
    }

    fn cast<'a, T: Value, F: FnOnce(raw::Local) -> T>(self, f: F) -> Handle<'a, T> {
        Handle::new_internal(f(self.to_raw()))
    }
}

#[cfg(feature = "legacy-runtime")]
#[repr(C)]
pub struct FunctionCallback<T: Value>(pub fn(FunctionContext) -> JsResult<T>);

#[cfg(feature = "legacy-runtime")]
impl<T: Value> Callback<()> for FunctionCallback<T> {
    extern "C" fn invoke(env: Env, info: CallbackInfo<'_>) {
        use crate::types::error::convert_panics;
        use crate::types::JsObject;

        unsafe {
            info.with_cx::<JsObject, _, _>(env, |cx| {
                let data = info.data(env);
                let dynamic_callback: fn(FunctionContext) -> JsResult<T> = std::mem::transmute(
                    neon_runtime::fun::get_dynamic_callback(env.to_raw(), data),
                );
                if let Ok(value) = convert_panics(env, || dynamic_callback(cx)) {
                    info.set_return(value);
                }
            })
        }
    }

    fn into_ptr(self) -> *mut c_void {
        self.0 as *mut _
    }
}

/// A dynamically computed callback that can be passed through C to the engine.
/// This type makes it possible to export a dynamically computed Rust function
/// as a pair of 1) a raw pointer to the dynamically computed function, and 2)
/// a static function that knows how to transmute that raw pointer and call it.
#[cfg(feature = "legacy-runtime")]
pub(crate) trait Callback<T: Clone + Copy + Sized>: Sized {
    /// Extracts the computed Rust function and invokes it. The Neon runtime
    /// ensures that the computed function is provided as the extra data field,
    /// wrapped as a V8 External, in the `CallbackInfo` argument.
    extern "C" fn invoke(env: Env, info: CallbackInfo<'_>) -> T;

    /// See `invoke`. This is used by the non-n-api implementation, so that every impl for this
    /// trait doesn't need to provide two versions of `invoke`.
    extern "C" fn invoke_compat(info: CallbackInfo<'_>) -> T {
        Self::invoke(Env::current(), info)
    }

    /// Converts the callback to a raw void pointer.
    fn into_ptr(self) -> *mut c_void;

    /// Exports the callback as a pair consisting of the static `Self::invoke`
    /// method and the computed callback, both converted to raw void pointers.
    fn into_c_callback(self) -> CCallback {
        CCallback {
            static_callback: unsafe { std::mem::transmute(Self::invoke_compat as usize) },
            dynamic_callback: self.into_ptr(),
        }
    }
}
