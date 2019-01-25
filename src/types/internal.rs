use std::mem;
use std::os::raw::c_void;
use neon_runtime;
use neon_runtime::raw;
use context::{CallbackInfo, FunctionContext};
use types::error::convert_panics;
use types::{JsObject, Managed};
use result::NeonResult;
use object::class::Callback;
use super::Value;

pub trait ValueInternal: Managed + 'static {
    fn name() -> String;

    fn is_typeof<Other: Value>(other: &Other) -> bool;

    fn downcast<Other: Value>(other: &Other) -> Option<&Self> {
        if Self::is_typeof(other) {
            Some(Self::from_raw(other.to_raw()))
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct FunctionCallback<T: Value>(pub fn(FunctionContext) -> NeonResult<&T>);

impl<T: Value> Callback<()> for FunctionCallback<T> {
    extern "C" fn invoke(info: &CallbackInfo) {
        unsafe {
            info.with_cx::<JsObject, _, _>(|mut cx| {
                let data = info.data(&mut cx);
                let dynamic_callback: fn(FunctionContext) -> NeonResult<&T> =
                    mem::transmute(neon_runtime::fun::get_dynamic_callback(data.to_raw()));
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
