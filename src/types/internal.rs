use super::Value;
use crate::context::{CallbackInfo, FunctionContext};
use crate::object::class::Callback;
use crate::result::JsResult;
use crate::types::error::convert_panics;
use crate::types::{Handle, JsObject, Managed};
use neon_runtime;
use neon_runtime::raw;
use std::mem;
use std::os::raw::c_void;

pub trait ValueInternal: Managed + 'static {
    fn name() -> String;

    fn is_typeof<Other: Value>(other: Other) -> bool;

    fn downcast<Other: Value>(other: Other) -> Option<Self> {
        if Self::is_typeof(other) {
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
pub struct FunctionCallback<T: Value>(pub fn(FunctionContext<'_>) -> JsResult<'_, T>);

impl<T: Value> Callback<()> for FunctionCallback<T> {
    extern "C" fn invoke(info: &CallbackInfo) {
        unsafe {
            info.with_cx::<JsObject, _, _>(|cx| {
                let data = info.data();
                let dynamic_callback: fn(FunctionContext<'_>) -> JsResult<'_, T> =
                    mem::transmute(neon_runtime::fun::get_dynamic_callback(data.to_raw()));
                if let Ok(value) = convert_panics(|| dynamic_callback(cx)) {
                    info.set_return(value);
                }
            })
        }
    }

    fn as_ptr(self) -> *mut c_void {
        unsafe { mem::transmute(self.0) }
    }
}
