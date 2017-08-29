use std::mem;
use std::ffi::CString;
use std::panic::{UnwindSafe, catch_unwind};

use neon_runtime;
use neon_runtime::raw;

use vm::{Throw, VmResult};
use js::{Value, Object, ToJsString, build};
use js::internal::ValueInternal;
use mem::{Handle, Managed};
use scope::Scope;

pub fn throw<'a, T: Value, U>(v: Handle<'a, T>) -> VmResult<U> {
    unsafe {
        neon_runtime::error::throw(v.to_raw());
    }
    Err(Throw)
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsError(raw::Local);

impl Managed for JsError {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsError(h) }
}

impl ValueInternal for JsError {
    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_error(other.to_raw()) }
    }
}

impl Value for JsError { }

impl Object for JsError { }

pub enum Kind {
    Error,
    TypeError,
    ReferenceError,
    RangeError,
    SyntaxError
}

fn message(msg: &str) -> CString {
    CString::new(msg).ok().unwrap_or_else(|| { CString::new("").ok().unwrap() })
}

impl JsError {
    pub fn new<'a, T: Scope<'a>, U: ToJsString>(scope: &mut T, kind: Kind, msg: U) -> VmResult<Handle<'a, JsError>> {
        let msg = msg.to_js_string(scope);
        build(|out| {
            unsafe {
                let raw = msg.to_raw();
                match kind {
                    Kind::Error          => neon_runtime::error::new_error(out, raw),
                    Kind::TypeError      => neon_runtime::error::new_type_error(out, raw),
                    Kind::ReferenceError => neon_runtime::error::new_reference_error(out, raw),
                    Kind::RangeError     => neon_runtime::error::new_range_error(out, raw),
                    Kind::SyntaxError    => neon_runtime::error::new_syntax_error(out, raw)
                }
            }
            true
        })
    }

    pub fn throw<T>(kind: Kind, msg: &str) -> VmResult<T> {
        let msg = &message(msg);
        unsafe {
            let ptr = mem::transmute(msg.as_ptr());
            match kind {
                Kind::Error          => neon_runtime::error::throw_error_from_cstring(ptr),
                Kind::TypeError      => neon_runtime::error::throw_type_error_from_cstring(ptr),
                Kind::ReferenceError => neon_runtime::error::throw_reference_error_from_cstring(ptr),
                Kind::RangeError     => neon_runtime::error::throw_range_error_from_cstring(ptr),
                Kind::SyntaxError    => neon_runtime::error::throw_syntax_error_from_cstring(ptr)
            }
        }
        Err(Throw)
    }
}

pub(crate) fn convert_panics<T, F: UnwindSafe + FnOnce() -> VmResult<T>>(f: F) -> VmResult<T> {
    match catch_unwind(|| { f() }) {
        Ok(result) => result,
        Err(panic) => {
            let msg = if let Some(string) = panic.downcast_ref::<String>() {
                format!("internal error in native module: {}", string)
            } else if let Some(str) = panic.downcast_ref::<&str>() {
                format!("internal error in native module: {}", str)
            } else {
                format!("internal error in native module")
            };
            JsError::throw::<T>(Kind::Error, &msg[..])
        }
    }
}
