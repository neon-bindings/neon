use std::mem;
use std::ffi::CString;

use neon_sys;
use neon_sys::raw;

use internal::vm::{Throw, VmResult};
use internal::js::{JsObject, Value, ValueInternal, Object, ToJsString, build};
use internal::mem::{Handle, Managed};
use scope::Scope;

pub fn throw<'a, T: Value, U>(v: Handle<'a, T>) -> VmResult<U> {
    unsafe {
        neon_sys::error::throw(v.to_raw());
    }
    Err(Throw)
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsTypeError(raw::Local);

impl Managed for JsTypeError {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsTypeError(h) }
}

impl ValueInternal for JsTypeError {
    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_type_error(other.to_raw()) }
    }
}

impl Value for JsTypeError { }

impl Object for JsTypeError { }

fn message(msg: &str) -> CString {
    CString::new(msg).ok().unwrap_or_else(|| { CString::new("").ok().unwrap() })
}

impl JsTypeError {
    pub fn new<'a, T: Scope<'a>, U: ToJsString>(scope: &mut T, msg: U) -> VmResult<Handle<'a, JsObject>> {
        let msg = msg.to_js_string(scope);
        build(|out| { unsafe { neon_sys::error::new_type_error(out, msg.to_raw()) } })
    }

    pub fn throw<T>(msg: &str) -> VmResult<T> {
        let msg = &message(msg);
        unsafe {
            neon_sys::error::throw_type_error_from_cstring(mem::transmute(msg.as_ptr()));
        }
        Err(Throw)
    }
}
