//! Types and traits representing JavaScript error values.

use std::panic::{catch_unwind, UnwindSafe};

use crate::{
    context::{internal::Env, Context},
    handle::{internal::TransparentNoCopyWrapper, Handle, Managed},
    object::Object,
    result::{NeonResult, Throw},
    sys::{self, raw},
    types::{build, private::ValueInternal, utf8::Utf8, Value},
};

/// The type of JavaScript
/// [`Error`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error)
/// objects.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// # fn test(mut cx: FunctionContext) -> JsResult<JsUndefined> {
/// // Create a type error:
/// let err = cx.type_error("expected a number, found a string")?;
///
/// // Add some custom diagnostic properties to the error:
/// let expected = cx.string("number");
/// err.set(&mut cx, "expected", expected)?;
///
/// let found = cx.string("string");
/// err.set(&mut cx, "found", found)?;
///
/// // Throw the error:
/// cx.throw(err)?;
/// # Ok(cx.undefined())
/// # }
/// ```
#[repr(transparent)]
#[derive(Debug)]
pub struct JsError(raw::Local);

unsafe impl TransparentNoCopyWrapper for JsError {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl Managed for JsError {
    fn to_raw(&self) -> raw::Local {
        self.0
    }

    fn from_raw(_: Env, h: raw::Local) -> Self {
        JsError(h)
    }
}

impl ValueInternal for JsError {
    fn name() -> String {
        "Error".to_string()
    }

    fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool {
        unsafe { sys::tag::is_error(env.to_raw(), other.to_raw()) }
    }
}

impl Value for JsError {}

impl Object for JsError {}

impl JsError {
    /// Creates a direct instance of the [`Error`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Error) class.
    ///
    /// **See also:** [`Context::error`]
    pub fn error<'a, C: Context<'a>, S: AsRef<str>>(
        cx: &mut C,
        msg: S,
    ) -> NeonResult<Handle<'a, JsError>> {
        let msg = cx.string(msg.as_ref());
        build(cx.env(), |out| unsafe {
            sys::error::new_error(cx.env().to_raw(), out, msg.to_raw());
            true
        })
    }

    /// Creates an instance of the [`TypeError`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/TypeError) class.
    ///
    /// **See also:** [`Context::type_error`]
    pub fn type_error<'a, C: Context<'a>, S: AsRef<str>>(
        cx: &mut C,
        msg: S,
    ) -> NeonResult<Handle<'a, JsError>> {
        let msg = cx.string(msg.as_ref());
        build(cx.env(), |out| unsafe {
            sys::error::new_type_error(cx.env().to_raw(), out, msg.to_raw());
            true
        })
    }

    /// Creates an instance of the [`RangeError`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/RangeError) class.
    ///
    /// **See also:** [`Context::range_error`]
    pub fn range_error<'a, C: Context<'a>, S: AsRef<str>>(
        cx: &mut C,
        msg: S,
    ) -> NeonResult<Handle<'a, JsError>> {
        let msg = cx.string(msg.as_ref());
        build(cx.env(), |out| unsafe {
            sys::error::new_range_error(cx.env().to_raw(), out, msg.to_raw());
            true
        })
    }
}

pub(crate) fn convert_panics<T, F: UnwindSafe + FnOnce() -> NeonResult<T>>(
    env: Env,
    f: F,
) -> NeonResult<T> {
    match catch_unwind(f) {
        Ok(result) => result,
        Err(panic) => {
            let msg = if let Some(string) = panic.downcast_ref::<String>() {
                format!("internal error in Neon module: {}", string)
            } else if let Some(str) = panic.downcast_ref::<&str>() {
                format!("internal error in Neon module: {}", str)
            } else {
                "internal error in Neon module".to_string()
            };
            let (data, len) = Utf8::from(&msg[..]).truncate().lower();
            unsafe {
                sys::error::clear_exception(env.to_raw());
                sys::error::throw_error_from_utf8(env.to_raw(), data, len);
                Err(Throw::new())
            }
        }
    }
}
