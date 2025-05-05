// See types_docs.rs for top-level module API docs.

#[cfg(feature = "napi-6")]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
pub mod bigint;
pub(crate) mod boxed;
pub mod buffer;
#[cfg(feature = "napi-5")]
pub(crate) mod date;
pub(crate) mod error;
pub mod extract;
pub mod function;
pub(crate) mod promise;

pub(crate) mod private;
pub(crate) mod utf8;

use std::{
    any,
    fmt::{self, Debug},
};

use private::prepare_call;
use smallvec::smallvec;

use crate::{
    context::{
        Context, Cx, FunctionContext,
        internal::{ContextInternal, Env},
    },
    handle::{
        Handle,
        internal::{SuperType, TransparentNoCopyWrapper},
    },
    object::Object,
    result::{JsResult, NeonResult, ResultExt, Throw},
    sys::{self, raw},
    types::{
        function::{BindOptions, CallOptions, ConstructOptions},
        private::ValueInternal,
        utf8::Utf8,
    },
};

pub use self::{
    boxed::{Finalize, JsBox},
    buffer::types::{
        JsArrayBuffer, JsBigInt64Array, JsBigUint64Array, JsBuffer, JsFloat32Array, JsFloat64Array,
        JsInt8Array, JsInt16Array, JsInt32Array, JsTypedArray, JsUint8Array, JsUint16Array,
        JsUint32Array,
    },
    error::JsError,
    promise::{Deferred, JsPromise},
};

#[cfg(feature = "napi-5")]
pub use self::date::{DateError, DateErrorKind, JsDate};

#[cfg(all(feature = "napi-5", feature = "futures"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "napi-5", feature = "futures"))))]
pub use self::promise::JsFuture;

// This should be considered deprecated and will be removed:
// https://github.com/neon-bindings/neon/issues/983
pub(crate) fn build<'a, T: Value, F: FnOnce(&mut raw::Local) -> bool>(
    env: Env,
    init: F,
) -> JsResult<'a, T> {
    unsafe {
        let mut local: raw::Local = std::mem::zeroed();
        if init(&mut local) {
            Ok(Handle::new_internal(T::from_local(env, local)))
        } else {
            Err(Throw::new())
        }
    }
}

impl<T: Value> SuperType<T> for JsValue {
    fn upcast_internal(v: &T) -> JsValue {
        JsValue(v.to_local())
    }
}

impl<T: Object> SuperType<T> for JsObject {
    fn upcast_internal(v: &T) -> JsObject {
        JsObject(v.to_local())
    }
}

/// The trait shared by all JavaScript values.
pub trait Value: ValueInternal {
    fn to_string<'cx, C: Context<'cx>>(&self, cx: &mut C) -> JsResult<'cx, JsString> {
        let env = cx.env();
        build(env, |out| unsafe {
            sys::convert::to_string(out, env.to_raw(), self.to_local())
        })
    }

    fn as_value<'cx, C: Context<'cx>>(&self, _: &mut C) -> Handle<'cx, JsValue> {
        JsValue::new_internal(self.to_local())
    }

    #[cfg(feature = "sys")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sys")))]
    /// Get a raw reference to the wrapped Node-API value.
    fn to_raw(&self) -> sys::Value {
        self.to_local()
    }

    #[cfg(feature = "sys")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sys")))]
    /// Creates a value from a raw Node-API value.
    ///
    /// # Safety
    ///
    /// * `value` must be of type `Self`
    /// * `value` must be valid for `'cx`
    unsafe fn from_raw<'cx, C: Context<'cx>>(cx: &C, value: sys::Value) -> Handle<'cx, Self> {
        Handle::new_internal(Self::from_local(cx.env(), value))
    }
}

/// The type of any JavaScript value, i.e., the root of all types.
///
/// The `JsValue` type is a catch-all type that sits at the top of the
/// [JavaScript type hierarchy](./index.html#the-javascript-type-hierarchy).
/// All JavaScript values can be safely and statically
/// [upcast](crate::handle::Handle::upcast) to `JsValue`; by contrast, a
/// [downcast](crate::handle::Handle::downcast) of a `JsValue` to another type
/// requires a runtime check.
/// (For TypeScript programmers, this can be thought of as similar to TypeScript's
/// [`unknown`](https://www.typescriptlang.org/docs/handbook/2/functions.html#unknown)
/// type.)
///
/// The `JsValue` type can be useful for generic, dynamic, or otherwise
/// hard-to-express API signatures, such as overloaded types:
///
/// ```
/// # use neon::prelude::*;
/// // Takes a string and adds the specified padding to the left.
/// // If the padding is a string, it's added as-is.
/// // If the padding is a number, then that number of spaces is added.
/// fn pad_left(mut cx: FunctionContext) -> JsResult<JsString> {
///     let string: Handle<JsString> = cx.argument(0)?;
///     let padding: Handle<JsValue> = cx.argument(1)?;
///
///     let padding: String = if let Ok(str) = padding.downcast::<JsString, _>(&mut cx) {
///         str.value(&mut cx)
///     } else if let Ok(num) = padding.downcast::<JsNumber, _>(&mut cx) {
///         " ".repeat(num.value(&mut cx) as usize)
///     } else {
///         return cx.throw_type_error("expected string or number");
///     };
///
///     let new_value = padding + &string.value(&mut cx);
///     Ok(cx.string(&new_value))
/// }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct JsValue(raw::Local);

impl Value for JsValue {}

unsafe impl TransparentNoCopyWrapper for JsValue {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl ValueInternal for JsValue {
    fn name() -> &'static str {
        "any"
    }

    fn is_typeof<Other: Value>(_cx: &mut Cx, _other: &Other) -> bool {
        true
    }

    fn to_local(&self) -> raw::Local {
        self.0
    }

    unsafe fn from_local(_env: Env, h: raw::Local) -> Self {
        JsValue(h)
    }
}

impl JsValue {
    pub(crate) fn new_internal<'a>(value: raw::Local) -> Handle<'a, JsValue> {
        Handle::new_internal(JsValue(value))
    }
}

/// The type of JavaScript
/// [`undefined`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures#primitive_values)
/// primitives.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// # fn test(mut cx: FunctionContext) -> JsResult<JsUndefined> {
/// // Extract the console object:
/// let console: Handle<JsObject> = cx.global("console")?;
///
/// // The undefined value:
/// let undefined = cx.undefined();
///
/// // Call console.log(undefined):
/// console.method(&mut cx, "log")?.arg(undefined)?.exec()?;
/// # Ok(undefined)
/// # }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct JsUndefined(raw::Local);

impl JsUndefined {
    /// Creates an `undefined` value.
    ///
    /// Although this method can be called many times, all `undefined`
    /// values are indistinguishable.
    ///
    /// **See also:** [`Context::undefined`]
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> Handle<'a, JsUndefined> {
        JsUndefined::new_internal(cx.env())
    }

    pub(crate) fn new_internal<'a>(env: Env) -> Handle<'a, JsUndefined> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            sys::primitive::undefined(&mut local, env.to_raw());
            Handle::new_internal(JsUndefined(local))
        }
    }
}

impl Value for JsUndefined {}

unsafe impl TransparentNoCopyWrapper for JsUndefined {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl ValueInternal for JsUndefined {
    fn name() -> &'static str {
        "undefined"
    }

    fn is_typeof<Other: Value>(cx: &mut Cx, other: &Other) -> bool {
        unsafe { sys::tag::is_undefined(cx.env().to_raw(), other.to_local()) }
    }

    fn to_local(&self) -> raw::Local {
        self.0
    }

    unsafe fn from_local(_env: Env, h: raw::Local) -> Self {
        JsUndefined(h)
    }
}

/// The type of JavaScript
/// [`null`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures#primitive_values)
/// primitives.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// # fn test(mut cx: FunctionContext) -> JsResult<JsNull> {
/// let null = cx.null();
/// cx.global::<JsObject>("console")?
///     .method(&mut cx, "log")?
///     .arg(null)?
///     .exec()?;
/// # Ok(null)
/// # }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct JsNull(raw::Local);

impl JsNull {
    /// Creates a `null` value.
    ///
    /// Although this method can be called many times, all `null`
    /// values are indistinguishable.
    ///
    /// **See also:** [`Context::null`]
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> Handle<'a, JsNull> {
        JsNull::new_internal(cx.env())
    }

    pub(crate) fn new_internal<'a>(env: Env) -> Handle<'a, JsNull> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            sys::primitive::null(&mut local, env.to_raw());
            Handle::new_internal(JsNull(local))
        }
    }
}

impl Value for JsNull {}

unsafe impl TransparentNoCopyWrapper for JsNull {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl ValueInternal for JsNull {
    fn name() -> &'static str {
        "null"
    }

    fn is_typeof<Other: Value>(cx: &mut Cx, other: &Other) -> bool {
        unsafe { sys::tag::is_null(cx.env().to_raw(), other.to_local()) }
    }

    fn to_local(&self) -> raw::Local {
        self.0
    }

    unsafe fn from_local(_env: Env, h: raw::Local) -> Self {
        JsNull(h)
    }
}

/// The type of JavaScript
/// [Boolean](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures#primitive_values)
/// primitives.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// # fn test(mut cx: FunctionContext) -> JsResult<JsUndefined> {
/// // The two Boolean values:
/// let t = cx.boolean(true);
/// let f = cx.boolean(false);
///
/// // Call console.log(true, false):
/// cx.global::<JsObject>("console")?.method(&mut cx, "log")?.args((t, f))?.exec()?;
/// # Ok(cx.undefined())
/// # }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct JsBoolean(raw::Local);

impl JsBoolean {
    /// Creates a Boolean value with value `b`.
    ///
    /// **See also:** [`Context::boolean`]
    pub fn new<'a, C: Context<'a>>(cx: &mut C, b: bool) -> Handle<'a, JsBoolean> {
        JsBoolean::new_internal(cx.env(), b)
    }

    pub(crate) fn new_internal<'a>(env: Env, b: bool) -> Handle<'a, JsBoolean> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            sys::primitive::boolean(&mut local, env.to_raw(), b);
            Handle::new_internal(JsBoolean(local))
        }
    }

    /// Returns the value of this Boolean as a Rust `bool`.
    pub fn value<'a, C: Context<'a>>(&self, cx: &mut C) -> bool {
        let env = cx.env().to_raw();
        unsafe { sys::primitive::boolean_value(env, self.to_local()) }
    }
}

impl Value for JsBoolean {}

unsafe impl TransparentNoCopyWrapper for JsBoolean {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl ValueInternal for JsBoolean {
    fn name() -> &'static str {
        "boolean"
    }

    fn is_typeof<Other: Value>(cx: &mut Cx, other: &Other) -> bool {
        unsafe { sys::tag::is_boolean(cx.env().to_raw(), other.to_local()) }
    }

    fn to_local(&self) -> raw::Local {
        self.0
    }

    unsafe fn from_local(_env: Env, h: raw::Local) -> Self {
        JsBoolean(h)
    }
}

/// The type of JavaScript
/// [string](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures#primitive_values)
/// primitives.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// # fn test(mut cx: FunctionContext) -> JsResult<JsUndefined> {
/// // Create a string:
/// let s = cx.string("hello 🥹");
///
/// // Call console.log(s):
/// cx.global::<JsObject>("console")?.method(&mut cx, "log")?.arg(s)?.exec()?;
/// # Ok(cx.undefined())
/// # }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct JsString(raw::Local);

/// An error produced when constructing a string that exceeds the limits of the runtime.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct StringOverflow(usize);

impl fmt::Display for StringOverflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "string size out of range: {}", self.0)
    }
}

/// The result of constructing a new `JsString`.
pub type StringResult<'a> = Result<Handle<'a, JsString>, StringOverflow>;

impl<'a> ResultExt<Handle<'a, JsString>> for StringResult<'a> {
    fn or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> JsResult<'a, JsString> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => cx.throw_range_error(e.to_string()),
        }
    }
}

impl Value for JsString {}

unsafe impl TransparentNoCopyWrapper for JsString {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl ValueInternal for JsString {
    fn name() -> &'static str {
        "string"
    }

    fn is_typeof<Other: Value>(cx: &mut Cx, other: &Other) -> bool {
        unsafe { sys::tag::is_string(cx.env().to_raw(), other.to_local()) }
    }

    fn to_local(&self) -> raw::Local {
        self.0
    }

    unsafe fn from_local(_env: Env, h: raw::Local) -> Self {
        JsString(h)
    }
}

impl JsString {
    /// Returns the size of the UTF-8 representation of this string,
    /// measured in 8-bit code units.
    ///
    /// Equivalent to `self.value(cx).len()` (but more efficient).
    ///
    /// # Example
    ///
    /// The string `"hello 🥹"` encodes as 10 bytes in UTF-8:
    ///
    /// - 6 bytes for `"hello "` (including the space).
    /// - 4 bytes for the emoji `"🥹"`.
    ///
    /// ```rust
    /// # use neon::prelude::*;
    /// # fn string_len(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// let str = cx.string("hello 🥹");
    /// assert_eq!(10, str.size(&mut cx));
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    pub fn size<'a, C: Context<'a>>(&self, cx: &mut C) -> usize {
        let env = cx.env().to_raw();

        unsafe { sys::string::utf8_len(env, self.to_local()) }
    }

    /// Returns the size of the UTF-16 representation of this string,
    /// measured in 16-bit code units.
    ///
    /// Equivalent to `self.to_utf16(cx).len()` (but more efficient).
    ///
    /// # Example
    ///
    /// The string `"hello 🥹"` encodes as 8 code units in UTF-16:
    ///
    /// - 6 `u16`s for `"hello "` (including the space).
    /// - 2 `u16`s for the emoji `"🥹"`.
    ///
    /// ```rust
    /// # use neon::prelude::*;
    /// # fn string_len_utf16(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// let str = cx.string("hello 🥹");
    /// assert_eq!(8, str.size_utf16(&mut cx));
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    pub fn size_utf16<'a, C: Context<'a>>(&self, cx: &mut C) -> usize {
        let env = cx.env().to_raw();

        unsafe { sys::string::utf16_len(env, self.to_local()) }
    }

    /// Convert this JavaScript string into a Rust [`String`].
    ///
    /// # Example
    ///
    /// This example function expects a single JavaScript string as argument
    /// and prints it out.
    ///
    /// ```rust
    /// # use neon::prelude::*;
    /// fn print_string(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    ///     let s = cx.argument::<JsString>(0)?.value(&mut cx);
    ///     println!("JavaScript string contents: {}", s);
    ///
    ///     Ok(cx.undefined())
    /// }
    /// ```
    pub fn value<'a, C: Context<'a>>(&self, cx: &mut C) -> String {
        let env = cx.env().to_raw();

        unsafe {
            let capacity = sys::string::utf8_len(env, self.to_local()) + 1;
            let mut buffer: Vec<u8> = Vec::with_capacity(capacity);
            let len = sys::string::data(env, buffer.as_mut_ptr(), capacity, self.to_local());
            buffer.set_len(len);
            String::from_utf8_unchecked(buffer)
        }
    }

    /// Convert this JavaScript string into a [`Vec<u16>`] encoded as UTF-16.
    ///
    /// The returned vector is guaranteed to be valid UTF-16, so libraries that handle
    /// UTF-16-encoded strings can assume the content to be valid.
    ///
    /// # Example
    ///
    /// This example function expects a single JavaScript string as argument and prints it out
    /// as a raw vector of `u16`s.
    ///
    /// ```rust
    /// # use neon::prelude::*;
    /// fn print_string_as_utf16(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    ///     let s = cx.argument::<JsString>(0)?.to_utf16(&mut cx);
    ///     println!("JavaScript string as raw UTF-16: {:?}", s);
    ///
    ///     Ok(cx.undefined())
    /// }
    /// ```
    ///
    /// This next example function also expects a single JavaScript string as argument and converts
    /// to a [`Vec<u16>`], but utilizes the [`widestring`](https://crates.io/crates/widestring)
    /// crate to handle the vector as a typical string.
    ///
    /// ```rust
    /// # use neon::prelude::*;
    /// use widestring::Utf16String;
    ///
    /// fn print_with_widestring(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    ///     let s = cx.argument::<JsString>(0)?.to_utf16(&mut cx);
    ///
    ///     // The returned vector is guaranteed to be valid UTF-16, so we can
    ///     // safely skip the validation step.
    ///     let s = unsafe { Utf16String::from_vec_unchecked(s) };
    ///
    ///     println!("JavaScript string as UTF-16: {}", s);
    ///
    ///     Ok(cx.undefined())
    /// }
    /// ```
    pub fn to_utf16<'a, C: Context<'a>>(&self, cx: &mut C) -> Vec<u16> {
        let env = cx.env().to_raw();

        unsafe {
            let capacity = sys::string::utf16_len(env, self.to_local()) + 1;
            let mut buffer: Vec<u16> = Vec::with_capacity(capacity);
            let len = sys::string::data_utf16(env, buffer.as_mut_ptr(), capacity, self.to_local());
            buffer.set_len(len);
            buffer
        }
    }

    /// Creates a new `JsString` value from a Rust string by copying its contents.
    ///
    /// This method panics if the string is longer than the maximum string size allowed
    /// by the JavaScript engine.
    ///
    /// # Example
    ///
    /// ```
    /// # use neon::prelude::*;
    /// # fn string_new(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// let str = JsString::new(&mut cx, "hello 🥹");
    /// assert_eq!(10, str.size(&mut cx));
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    ///
    /// **See also:** [`Context::string`]
    pub fn new<'a, C: Context<'a>, S: AsRef<str>>(cx: &mut C, val: S) -> Handle<'a, JsString> {
        JsString::try_new(cx, val).unwrap()
    }

    /// Tries to create a new `JsString` value from a Rust string by copying its contents.
    ///
    /// Returns `Err(StringOverflow)` if the string is longer than the maximum string size
    /// allowed by the JavaScript engine.
    ///
    /// # Example
    ///
    /// This example tries to construct a JavaScript string from a Rust string of
    /// unknown length, and on overflow generates an alternate truncated string with
    /// a suffix (`"[…]"`) to indicate the truncation.
    ///
    /// ```
    /// # use neon::prelude::*;
    /// # fn string_try_new(mut cx: FunctionContext) -> JsResult<JsString> {
    /// # static str: &'static str = "hello 🥹";
    /// let s = match JsString::try_new(&mut cx, str) {
    ///     Ok(s) => s,
    ///     Err(_) => cx.string(format!("{}[…]", &str[0..32])),
    /// };
    /// # Ok(s)
    /// # }
    /// ```
    pub fn try_new<'a, C: Context<'a>, S: AsRef<str>>(cx: &mut C, val: S) -> StringResult<'a> {
        let val = val.as_ref();
        match JsString::new_internal(cx.env(), val) {
            Some(s) => Ok(s),
            None => Err(StringOverflow(val.len())),
        }
    }

    pub(crate) fn new_internal<'a>(env: Env, val: &str) -> Option<Handle<'a, JsString>> {
        let (ptr, len) = if let Some(small) = Utf8::from(val).into_small() {
            small.lower()
        } else {
            return None;
        };

        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            if sys::string::new(&mut local, env.to_raw(), ptr, len) {
                Some(Handle::new_internal(JsString(local)))
            } else {
                None
            }
        }
    }
}

/// The type of JavaScript
/// [number](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures#primitive_values)
/// primitives.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// # fn test(mut cx: FunctionContext) -> JsResult<JsUndefined> {
/// // Create a number:
/// let n = cx.number(17.0);
///
/// // Call console.log(n):
/// cx.global::<JsObject>("console")?.method(&mut cx, "log")?.arg(n)?.exec()?;
/// # Ok(cx.undefined())
/// # }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct JsNumber(raw::Local);

impl JsNumber {
    /// Creates a new number with value `x`.
    ///
    /// **See also:** [`Context::number`]
    pub fn new<'a, C: Context<'a>, T: Into<f64>>(cx: &mut C, x: T) -> Handle<'a, JsNumber> {
        JsNumber::new_internal(cx.env(), x.into())
    }

    pub(crate) fn new_internal<'a>(env: Env, v: f64) -> Handle<'a, JsNumber> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            sys::primitive::number(&mut local, env.to_raw(), v);
            Handle::new_internal(JsNumber(local))
        }
    }

    /// Returns the value of this number as a Rust `f64`.
    pub fn value<'a, C: Context<'a>>(&self, cx: &mut C) -> f64 {
        let env = cx.env().to_raw();
        unsafe { sys::primitive::number_value(env, self.to_local()) }
    }
}

impl Value for JsNumber {}

unsafe impl TransparentNoCopyWrapper for JsNumber {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl ValueInternal for JsNumber {
    fn name() -> &'static str {
        "number"
    }

    fn is_typeof<Other: Value>(cx: &mut Cx, other: &Other) -> bool {
        unsafe { sys::tag::is_number(cx.env().to_raw(), other.to_local()) }
    }

    fn to_local(&self) -> raw::Local {
        self.0
    }

    unsafe fn from_local(_env: Env, h: raw::Local) -> Self {
        JsNumber(h)
    }
}

/// The type of JavaScript
/// [objects](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures#objects),
/// i.e., the root of all object types.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// # fn test(mut cx: FunctionContext) -> JsResult<JsUndefined> {
/// // Create an object:
/// let obj = cx.empty_object()
///     .prop(&mut cx, "name")
///     .set("Neon")?
///     .prop("url")
///     .set("https://neon-bindings.com")?
///     .this();
///
/// // Call console.log(obj):
/// cx.global::<JsObject>("console")?.method(&mut cx, "log")?.arg(obj)?.exec()?;
/// # Ok(cx.undefined())
/// # }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct JsObject(raw::Local);

impl Value for JsObject {}

unsafe impl TransparentNoCopyWrapper for JsObject {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl ValueInternal for JsObject {
    fn name() -> &'static str {
        "object"
    }

    fn is_typeof<Other: Value>(cx: &mut Cx, other: &Other) -> bool {
        unsafe { sys::tag::is_object(cx.env().to_raw(), other.to_local()) }
    }

    fn to_local(&self) -> raw::Local {
        self.0
    }

    unsafe fn from_local(_env: Env, h: raw::Local) -> Self {
        JsObject(h)
    }
}

impl Object for JsObject {}

impl JsObject {
    /// Creates a new empty object.
    ///
    /// **See also:** [`Context::empty_object`]
    pub fn new<'a, C: Context<'a>>(c: &mut C) -> Handle<'a, JsObject> {
        JsObject::new_internal(c.env())
    }

    pub(crate) fn new_internal<'a>(env: Env) -> Handle<'a, JsObject> {
        JsObject::build(|out| unsafe { sys::object::new(out, env.to_raw()) })
    }

    pub(crate) fn build<'a, F: FnOnce(&mut raw::Local)>(init: F) -> Handle<'a, JsObject> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            init(&mut local);
            Handle::new_internal(JsObject(local))
        }
    }
}

/// The type of JavaScript
/// [`Array`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array)
/// objects.
///
/// An array is any JavaScript value for which
/// [`Array.isArray`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/isArray)
/// would return `true`.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsArray> {
/// // Create a new empty array:
/// let a: Handle<JsArray> = cx.empty_array();
///
/// // Push some values onto the array:
/// a.prop(&mut cx, 0).set(17)?;
/// a.prop(&mut cx, 1).set("hello")?;
/// # Ok(a)
/// # }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct JsArray(raw::Local);

impl JsArray {
    /// Constructs a new empty array of length `len`, equivalent to the JavaScript
    /// expression `new Array(len)`.
    ///
    /// Note that for non-zero `len`, this creates a
    /// [sparse array](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Indexed_collections#sparse_arrays),
    /// which can sometimes have surprising behavior. To ensure that a new array
    /// is and remains dense (i.e., not sparse), consider creating an empty array
    /// with `JsArray::new(cx, 0)` or `cx.empty_array()` and only appending
    /// elements to the end of the array.
    ///
    /// **See also:** [`Context::empty_array`]
    pub fn new<'a, C: Context<'a>>(cx: &mut C, len: usize) -> Handle<'a, JsArray> {
        JsArray::new_internal(cx.env(), len)
    }

    pub(crate) fn new_internal<'a>(env: Env, len: usize) -> Handle<'a, JsArray> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            sys::array::new(&mut local, env.to_raw(), len);
            Handle::new_internal(JsArray(local))
        }
    }

    /// Copies the array contents into a new [`Vec`] by iterating through all indices
    /// from 0 to `self.len()`.
    ///
    /// The length is dynamically checked on each iteration in case the array is modified
    /// during the computation.
    pub fn to_vec<'a, C: Context<'a>>(&self, cx: &mut C) -> NeonResult<Vec<Handle<'a, JsValue>>> {
        let mut result = Vec::with_capacity(self.len_inner(cx.env()) as usize);
        let mut i = 0;
        loop {
            // Since getting a property can trigger arbitrary code,
            // we have to re-check the length on every iteration.
            if i >= self.len_inner(cx.env()) {
                return Ok(result);
            }
            result.push(self.get(cx, i)?);
            i += 1;
        }
    }

    fn len_inner(&self, env: Env) -> u32 {
        unsafe { sys::array::len(env.to_raw(), self.to_local()) }
    }

    #[allow(clippy::len_without_is_empty)]
    /// Returns the length of the array, equivalent to the JavaScript expression
    /// [`this.length`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/length).
    pub fn len<'a, C: Context<'a>>(&self, cx: &mut C) -> u32 {
        self.len_inner(cx.env())
    }

    /// Indicates whether the array is empty, equivalent to
    /// `self.len() == 0`.
    pub fn is_empty<'a, C: Context<'a>>(&self, cx: &mut C) -> bool {
        self.len(cx) == 0
    }
}

impl Value for JsArray {}

unsafe impl TransparentNoCopyWrapper for JsArray {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl ValueInternal for JsArray {
    fn name() -> &'static str {
        "Array"
    }

    fn is_typeof<Other: Value>(cx: &mut Cx, other: &Other) -> bool {
        unsafe { sys::tag::is_array(cx.env().to_raw(), other.to_local()) }
    }

    fn to_local(&self) -> raw::Local {
        self.0
    }

    unsafe fn from_local(_env: Env, h: raw::Local) -> Self {
        JsArray(h)
    }
}

impl Object for JsArray {}

/// The type of JavaScript
/// [`Function`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function)
/// objects.
#[derive(Debug)]
#[repr(transparent)]
///
/// A `JsFunction` may come from an existing JavaScript function, for example
/// by extracting it from the property of another object such as the
/// [global object](crate::context::Context::global), or it may be defined in Rust
/// with [`JsFunction::new()`](JsFunction::new).
///
/// ## Calling functions
///
/// Neon provides a convenient syntax for calling JavaScript functions with the
/// [`bind()`](JsFunction::bind) method, which produces a [`BindOptions`](BindOptions)
/// struct that can be used to provide the function arguments (and optionally, the binding for
/// `this`) before calling the function:
/// ```
/// # use neon::prelude::*;
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsNumber> {
/// // Extract the parseInt function from the global object
/// let parse_int: Handle<JsFunction> = cx.global("parseInt")?;
///
/// // Call parseInt("42")
/// let x: Handle<JsNumber> = parse_int
///     .bind(&mut cx)
///     .arg("42")?
///     .call()?;
/// # Ok(x)
/// # }
/// ```
///
/// ## Calling functions as constructors
///
/// A `JsFunction` can be called as a constructor (like `new Array(16)` or
/// `new URL("https://neon-bindings.com")`) with the
/// [`construct()`](BindOptions::construct) method:
/// ```
/// # use neon::prelude::*;
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsObject> {
/// // Extract the URL constructor from the global object
/// let url: Handle<JsFunction> = cx.global("URL")?;
///
/// // Call new URL("https://neon-bindings.com")
/// let obj = url
///     .bind(&mut cx)
///     .arg("https://neon-bindings.com")?
///     .construct()?;
/// # Ok(obj)
/// # }
/// ```
///
/// ## Defining functions
///
/// JavaScript functions can be defined in Rust with the
/// [`JsFunction::new()`](JsFunction::new) constructor, which takes
/// a Rust implementation function and produces a JavaScript function.
///
/// ```
/// # use neon::prelude::*;
/// // A function implementation that adds 1 to its first argument
/// fn add1(mut cx: FunctionContext) -> JsResult<JsNumber> {
///     let x: Handle<JsNumber> = cx.argument(0)?;
///     let v = x.value(&mut cx);
///     Ok(cx.number(v + 1.0))
/// }
///
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsFunction> {
/// // Define a new JsFunction implemented with the add1 function
/// let f = JsFunction::new(&mut cx, add1)?;
/// # Ok(f)
/// # }
/// ```
pub struct JsFunction {
    raw: raw::Local,
}

impl Object for JsFunction {}

impl JsFunction {
    #[cfg(not(feature = "napi-5"))]
    /// Returns a new `JsFunction` implemented by `f`.
    pub fn new<'a, C, U>(
        cx: &mut C,
        f: fn(FunctionContext) -> JsResult<U>,
    ) -> JsResult<'a, JsFunction>
    where
        C: Context<'a>,
        U: Value,
    {
        let name = any::type_name::<F>();

        Self::new_internal(cx, f, name)
    }

    #[cfg(feature = "napi-5")]
    /// Returns a new `JsFunction` implemented by `f`.
    pub fn new<'a, C, F, V>(cx: &mut C, f: F) -> JsResult<'a, JsFunction>
    where
        C: Context<'a>,
        F: Fn(FunctionContext) -> JsResult<V> + 'static,
        V: Value,
    {
        let name = any::type_name::<F>();

        Self::new_internal(cx, f, name)
    }

    #[cfg(not(feature = "napi-5"))]
    /// Returns a new `JsFunction` implemented by `f` with specified name
    pub fn with_name<'a, C, U>(
        cx: &mut C,
        name: &str,
        f: fn(FunctionContext) -> JsResult<U>,
    ) -> JsResult<'a, JsFunction>
    where
        C: Context<'a>,
        U: Value,
    {
        Self::new_internal(cx, f, name)
    }

    #[cfg(feature = "napi-5")]
    /// Returns a new `JsFunction` implemented by `f` with specified name
    pub fn with_name<'a, C, F, V>(cx: &mut C, name: &str, f: F) -> JsResult<'a, JsFunction>
    where
        C: Context<'a>,
        F: Fn(FunctionContext) -> JsResult<V> + 'static,
        V: Value,
    {
        Self::new_internal(cx, f, name)
    }

    fn new_internal<'a, C, F, V>(cx: &mut C, f: F, name: &str) -> JsResult<'a, JsFunction>
    where
        C: Context<'a>,
        F: Fn(FunctionContext) -> JsResult<V> + 'static,
        V: Value,
    {
        use std::panic::AssertUnwindSafe;
        use std::ptr;

        use crate::context::CallbackInfo;
        use crate::types::error::convert_panics;

        let f = move |env: raw::Env, info| {
            let env = env.into();
            let info = unsafe { CallbackInfo::new(info) };

            FunctionContext::with(env, &info, |cx| {
                convert_panics(env, AssertUnwindSafe(|| f(cx)))
                    .map(|v| v.to_local())
                    // We do not have a Js Value to return, most likely due to an exception.
                    // If we are in a throwing state, constructing a Js Value would be invalid.
                    // While not explicitly written, the Node-API documentation includes many examples
                    // of returning `NULL` when a native function does not return a value.
                    // https://nodejs.org/api/n-api.html#n_api_napi_create_function
                    .unwrap_or_else(|_: Throw| ptr::null_mut())
            })
        };

        unsafe {
            if let Ok(raw) = sys::fun::new(cx.env().to_raw(), name, f) {
                Ok(Handle::new_internal(JsFunction { raw }))
            } else {
                Err(Throw::new())
            }
        }
    }
}

impl JsFunction {
    /// Calls this function.
    ///
    /// **See also:** [`JsFunction::bind`].
    pub fn call<'a, 'b, C: Context<'a>, T, AS>(
        &self,
        cx: &mut C,
        this: Handle<'b, T>,
        args: AS,
    ) -> JsResult<'a, JsValue>
    where
        T: Value,
        AS: AsRef<[Handle<'b, JsValue>]>,
    {
        unsafe { self.try_call(cx, this, args) }
    }

    /// Calls this function for side effect, discarding its result.
    ///
    /// **See also:** [`JsFunction::bind`].
    pub fn exec<'a, 'b, C: Context<'a>, T, AS>(
        &self,
        cx: &mut C,
        this: Handle<'b, T>,
        args: AS,
    ) -> NeonResult<()>
    where
        T: Value,
        AS: AsRef<[Handle<'b, JsValue>]>,
    {
        self.call(cx, this, args)?;
        Ok(())
    }

    /// Calls this function as a constructor.
    ///
    /// **See also:** [`JsFunction::bind`].
    pub fn construct<'a, 'b, C: Context<'a>, AS>(
        &self,
        cx: &mut C,
        args: AS,
    ) -> JsResult<'a, JsObject>
    where
        AS: AsRef<[Handle<'b, JsValue>]>,
    {
        let (argc, argv) = unsafe { prepare_call(cx, args.as_ref()) }?;
        let env = cx.env().to_raw();
        build(cx.env(), |out| unsafe {
            sys::fun::construct(out, env, self.to_local(), argc, argv)
        })
    }
}

impl JsFunction {
    /// Create a [`BindOptions`] builder for calling this function.
    ///
    /// The builder methods make it convenient to assemble the call from parts:
    /// ```
    /// # use neon::prelude::*;
    /// # fn foo(mut cx: FunctionContext) -> JsResult<JsNumber> {
    /// # let parse_int: Handle<JsFunction> = cx.global("parseInt")?;
    /// let x: f64 = parse_int
    ///     .bind(&mut cx)
    ///     .arg("42")?
    ///     .call()?;
    /// # Ok(cx.number(x))
    /// # }
    /// ```
    pub fn bind<'a, 'cx: 'a>(&self, cx: &'a mut Cx<'cx>) -> BindOptions<'a, 'cx> {
        let callee = self.as_value(cx);
        BindOptions {
            cx,
            callee,
            this: None,
            args: smallvec![],
        }
    }
}

impl JsFunction {
    /// Create a [`CallOptions`](function::CallOptions) for calling this function.
    #[deprecated(since = "TBD", note = "use `JsFunction::bind` instead")]
    pub fn call_with<'a, C: Context<'a>>(&self, _cx: &C) -> CallOptions<'a> {
        CallOptions {
            this: None,
            // # Safety
            // Only a single context may be used at a time because parent scopes
            // are locked with `&mut self`. Therefore, the lifetime of `CallOptions`
            // will always be the most narrow scope possible.
            callee: Handle::new_internal(unsafe { self.clone() }),
            args: smallvec![],
        }
    }

    /// Create a [`ConstructOptions`](function::ConstructOptions) for calling this function
    /// as a constructor.
    #[deprecated(since = "TBD", note = "use `JsFunction::bind` instead")]
    pub fn construct_with<'a, C: Context<'a>>(&self, _cx: &C) -> ConstructOptions<'a> {
        ConstructOptions {
            // # Safety
            // Only a single context may be used at a time because parent scopes
            // are locked with `&mut self`. Therefore, the lifetime of `ConstructOptions`
            // will always be the most narrow scope possible.
            callee: Handle::new_internal(unsafe { self.clone() }),
            args: smallvec![],
        }
    }

    /// # Safety
    /// The caller must wrap in a `Handle` with an appropriate lifetime.
    unsafe fn clone(&self) -> Self {
        Self { raw: self.raw }
    }
}

impl Value for JsFunction {}

unsafe impl TransparentNoCopyWrapper for JsFunction {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.raw
    }
}

impl ValueInternal for JsFunction {
    fn name() -> &'static str {
        "function"
    }

    fn is_typeof<Other: Value>(cx: &mut Cx, other: &Other) -> bool {
        unsafe { sys::tag::is_function(cx.env().to_raw(), other.to_local()) }
    }

    fn to_local(&self) -> raw::Local {
        self.raw
    }

    unsafe fn from_local(_env: Env, h: raw::Local) -> Self {
        JsFunction { raw: h }
    }
}

#[cfg(feature = "napi-6")]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
#[derive(Debug)]
#[repr(transparent)]
/// The type of JavaScript
/// [`BigInt`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt)
/// values.
///
/// # Example
///
/// The following shows an example of adding two numbers that exceed
/// [`Number.MAX_SAFE_INTEGER`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER).
///
/// ```
/// # use neon::{prelude::*, types::JsBigInt};
///
/// fn add_bigint(mut cx: FunctionContext) -> JsResult<JsBigInt> {
///     // Get references to the `BigInt` arguments
///     let a = cx.argument::<JsBigInt>(0)?;
///     let b = cx.argument::<JsBigInt>(1)?;
///
///     // Convert the `BigInt` to `i64`
///     let a = a.to_i64(&mut cx)
///         // On failure, convert err to a `RangeError` exception
///         .or_throw(&mut cx)?;
///
///     let b = b.to_i64(&mut cx).or_throw(&mut cx)?;
///     let sum = a + b;
///
///     // Create a `BigInt` from the `i64` sum
///     Ok(JsBigInt::from_i64(&mut cx, sum))
/// }
/// ```
pub struct JsBigInt(raw::Local);
