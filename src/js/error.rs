//! Types and traits representing JavaScript error values.

use std::mem;
use std::ffi::CString;
use std::panic::{UnwindSafe, catch_unwind};

use neon_runtime;
use neon_runtime::raw;

use vm::{Throw, Context, VmResult, Handle, Managed};
use js::{Value, Object, ToJsString, build};
use js::internal::ValueInternal;

/// A JS `Error` object.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsError(raw::Local);

impl Managed for JsError {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsError(h) }
}

impl ValueInternal for JsError {
    fn name() -> String { "Error".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_error(other.to_raw()) }
    }
}

impl Value for JsError { }

impl Object for JsError { }

/// Distinguishes between the different standard JS subclasses of `Error`.
pub enum ErrorKind {

    /// Represents a direct instance of the [`Error`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Error) class.
    Error,

    /// Represents an instance of the [`TypeError`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/TypeError) class.
    TypeError,

    /// Represents an instance of the [`ReferenceError`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/ReferenceError) class.
    ReferenceError,

    /// Represents an instance of the [`RangeError`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/RangeError) class.
    RangeError,

    /// Represents an instance of the [`SyntaxError`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/SyntaxError) class.
    SyntaxError

}

fn message(msg: &str) -> CString {
    CString::new(msg).ok().unwrap_or_else(|| { CString::new("").ok().unwrap() })
}

impl JsError {

    /// Constructs a new error object.
    pub fn new<'a, C: Context<'a>, U: ToJsString>(cx: &mut C, kind: ErrorKind, msg: U) -> VmResult<Handle<'a, JsError>> {
        let msg = msg.to_js_string(cx);
        build(|out| {
            unsafe {
                let raw = msg.to_raw();
                match kind {
                    ErrorKind::Error          => neon_runtime::error::new_error(out, raw),
                    ErrorKind::TypeError      => neon_runtime::error::new_type_error(out, raw),
                    ErrorKind::ReferenceError => neon_runtime::error::new_reference_error(out, raw),
                    ErrorKind::RangeError     => neon_runtime::error::new_range_error(out, raw),
                    ErrorKind::SyntaxError    => neon_runtime::error::new_syntax_error(out, raw)
                }
            }
            true
        })
    }

    /// Convenience method for throwing a new error object.
    pub fn throw<'a, C: Context<'a>, T>(_: &mut C, kind: ErrorKind, msg: &str) -> VmResult<T> {
        unsafe {
            throw_new(kind, msg)
        }
    }

}

unsafe fn throw_new<T>(kind: ErrorKind, msg: &str) -> VmResult<T> {
    let msg = &message(msg);
    let ptr = mem::transmute(msg.as_ptr());
    match kind {
        ErrorKind::Error          => neon_runtime::error::throw_error_from_cstring(ptr),
        ErrorKind::TypeError      => neon_runtime::error::throw_type_error_from_cstring(ptr),
        ErrorKind::ReferenceError => neon_runtime::error::throw_reference_error_from_cstring(ptr),
        ErrorKind::RangeError     => neon_runtime::error::throw_range_error_from_cstring(ptr),
        ErrorKind::SyntaxError    => neon_runtime::error::throw_syntax_error_from_cstring(ptr)
    }
    Err(Throw)
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
            unsafe {
                throw_new::<T>(ErrorKind::Error, &msg[..])
            }
        }
    }
}
