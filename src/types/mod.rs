//! Representations of JavaScript's core builtin types.

pub(crate) mod binary;
pub(crate) mod error;

pub(crate) mod internal;
pub(crate) mod utf8;

use std;
use std::error::Error;
use std::fmt;
use std::mem;
use std::os::raw::c_void;
use std::marker::PhantomData;
use neon_runtime;
use neon_runtime::raw;
use context::{Context, FunctionContext};
use result::{NeonResult, NeonResultExt};
use object::{Object, This};
use object::class::Callback;
use self::internal::{ValueInternal, FunctionCallback};
use self::utf8::Utf8;

pub use self::binary::{JsBuffer, JsArrayBuffer, BinaryData, BinaryViewType};
pub use self::error::JsError;

pub unsafe trait SuperType<T: Value> { }

unsafe impl<T: Value> SuperType<T> for JsValue { }

unsafe impl<T: Object> SuperType<T> for JsObject { }

/// The trait of data that is managed by the JS garbage collector and can only be accessed via handles.
pub trait Managed: Sized {
    fn to_raw(&self) -> &raw::Persistent {
        unsafe {
            std::mem::transmute(self)
        }
    }

    fn from_raw(h: &raw::Persistent) -> &Self {
        unsafe {
            std::mem::transmute(h)
        }
    }
}

pub trait Value: ValueInternal {

    /// Safely upcast a JS value to a supertype.
    /// 
    /// This method does not require an execution context because it only reinterprets a pointer.
    fn upcast<U: Value + SuperType<Self>>(&self) -> &U {
        unsafe {
            mem::transmute(self)
        }
    }


    /// Tests whether this value is an instance of the given type.
    /// 
    /// # Example:
    /// 
    /// ```no_run
    /// # use neon::prelude::*;
    /// # fn my_neon_function(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// let v: Handle<JsValue> = cx.number(17).upcast();
    /// v.is_a::<JsString>(); // false
    /// v.is_a::<JsNumber>(); // true
    /// v.is_a::<JsValue>();  // true
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    fn is_a<U: Value>(&self) -> bool {
        U::is_typeof(self)
    }

    /// Attempts to downcast a handle to another type, which may fail. A failure
    /// to downcast **does not** throw a JavaScript exception, so it's OK to
    /// continue interacting with the JS engine if this method produces an `Err`
    /// result.
    fn downcast<U: Value>(&self) -> DowncastResult<Self, U> {
        if U::is_typeof(self) {
            Ok(unsafe { mem::transmute(self) })
        } else {
            Err(DowncastError::new())
        }
    }

    /// Attempts to downcast a handle to another type, raising a JavaScript `TypeError`
    /// exception on failure. This method is a convenient shorthand, equivalent to
    /// `self.downcast::<U>().or_throw::<C>(cx)`.
    fn downcast_or_throw<'b, U: Value, C: Context<'b>>(&self, cx: &mut C) -> NeonResult<&U> {
        self.downcast().or_throw(cx)
    }

}

/// An error representing a failed downcast.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct DowncastError<F: Value, T: Value> {
    phantom_from: PhantomData<F>,
    phantom_to: PhantomData<T>,
    description: String
}

impl<F: Value, T: Value> fmt::Debug for DowncastError<F, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "DowncastError")
    }
}

impl<F: Value, T: Value> DowncastError<F, T> {
    fn new() -> Self {
        DowncastError {
            phantom_from: PhantomData,
            phantom_to: PhantomData,
            description: format!("failed downcast to {}", T::name())
        }
    }
}

impl<F: Value, T: Value> fmt::Display for DowncastError<F, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl<F: Value, T: Value> Error for DowncastError<F, T> {
    fn description(&self) -> &str {
        &self.description
    }
}

/// The result of a call to `Handle::downcast()`.
pub type DowncastResult<'a, F, T> = Result<&'a T, DowncastError<F, T>>;

impl<'a, F: Value, T: Value> NeonResultExt<'a, T> for DowncastResult<'a, F, T> {
    fn or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> NeonResult<&'a T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => cx.throw_type_error(&e.description)
        }
    }
}

/// A JavaScript value of any type.
#[repr(C)]
pub struct JsValue(raw::Persistent);

impl Value for JsValue { }

impl Managed for JsValue { }

impl ValueInternal for JsValue {
    fn name() -> String { "any".to_string() }

    fn is_typeof<Other: Value>(_: &Other) -> bool {
        true
    }
}

unsafe impl This for JsValue { }

/// The JavaScript `undefined` value.
#[repr(C)]
pub struct JsUndefined(raw::Persistent);

impl JsUndefined {
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> &'a JsUndefined {
        cx.new_infallible(|out, isolate| unsafe {
            neon_runtime::primitive::undefined(out, isolate)
        })
    }
}

impl Managed for JsUndefined { }

impl ValueInternal for JsUndefined {
    fn name() -> String { "undefined".to_string() }

    fn is_typeof<Other: Value>(other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_undefined(other.to_raw()) }
    }
}

impl Value for JsUndefined { }

unsafe impl This for JsUndefined { }

/// The JavaScript `null` value.
#[repr(C)]
pub struct JsNull(raw::Persistent);

impl JsNull {
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> &'a JsNull {
        cx.new_infallible(|out, isolate| unsafe {
            neon_runtime::primitive::null(out, isolate)
        })
    }
}

impl Managed for JsNull { }

impl ValueInternal for JsNull {
    fn name() -> String { "null".to_string() }

    fn is_typeof<Other: Value>(other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_null(other.to_raw()) }
    }
}

impl Value for JsNull { }

/// A JavaScript boolean primitive value.
#[repr(C)]
pub struct JsBoolean(raw::Persistent);

impl JsBoolean {
    pub fn new<'a, C: Context<'a>>(cx: &mut C, b: bool) -> &'a JsBoolean {
        cx.new_infallible(|out, isolate| unsafe {
            neon_runtime::primitive::boolean(out, isolate, b)
        })
    }

    pub fn value(&self) -> bool {
        unsafe {
            neon_runtime::primitive::boolean_value(self.to_raw())
        }
    }
}

impl Value for JsBoolean { }

impl Managed for JsBoolean { }

impl ValueInternal for JsBoolean {
    fn name() -> String { "boolean".to_string() }

    fn is_typeof<Other: Value>(other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_boolean(other.to_raw()) }
    }
}

/// A JavaScript string primitive value.
#[repr(C)]
pub struct JsString(raw::Persistent);

/// An error produced when constructing a string that exceeds the JS engine's maximum string size.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct StringOverflow(usize);

impl fmt::Display for StringOverflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "string size out of range: {}", self.0)
    }
}

/// The result of constructing a new `JsString`.
pub type StringResult<'a> = Result<&'a JsString, StringOverflow>;

impl<'a> NeonResultExt<'a, JsString> for StringResult<'a> {
    fn or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> NeonResult<&'a JsString> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => cx.throw_range_error(&e.to_string())
        }
    }
}

impl Value for JsString { }

impl Managed for JsString { }

impl ValueInternal for JsString {
    fn name() -> String { "string".to_string() }

    fn is_typeof<Other: Value>(other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_string(other.to_raw()) }
    }
}

impl JsString {
    pub fn size(self) -> isize {
        unsafe {
            neon_runtime::string::utf8_len(self.to_raw())
        }
    }

    pub fn value(self) -> String {
        unsafe {
            let capacity = neon_runtime::string::utf8_len(self.to_raw());
            let mut buffer: Vec<u8> = Vec::with_capacity(capacity as usize);
            let p = buffer.as_mut_ptr();
            std::mem::forget(buffer);
            let len = neon_runtime::string::data(p, capacity, self.to_raw());
            String::from_raw_parts(p, len as usize, capacity as usize)
        }
    }

    pub fn new<'a, C: Context<'a>, S: AsRef<str>>(cx: &mut C, val: S) -> &'a JsString {
        JsString::try_new(cx, val).unwrap()
    }

    pub fn try_new<'a, C: Context<'a>, S: AsRef<str>>(cx: &mut C, val: S) -> StringResult<'a> {
        let val = val.as_ref();
        match JsString::new_internal(cx, val) {
            Some(s) => Ok(s),
            None => Err(StringOverflow(val.len()))
        }
    }

    fn new_internal<'a, C: Context<'a>>(cx: &mut C, val: &str) -> Option<&'a JsString> {
        Utf8::from(val)
            .into_small()
            .and_then(|small| {
                let (ptr, len) = small.lower();
                cx.new_opt(|out, isolate| unsafe {
                    neon_runtime::string::new(out, isolate, ptr, len)
                })
            })
    }
}

/// A JavaScript number value.
#[repr(C)]
pub struct JsNumber(raw::Persistent);

impl JsNumber {
    pub fn new<'a, C: Context<'a>, T: Into<f64>>(cx: &mut C, x: T) -> &'a JsNumber {
        cx.new_infallible(|out, isolate| unsafe {
            neon_runtime::primitive::number(out, isolate, x.into())
        })
    }

    pub fn value(&self) -> f64 {
        unsafe {
            neon_runtime::primitive::number_value(self.to_raw())
        }
    }
}

impl Managed for JsNumber { }

impl ValueInternal for JsNumber {
    fn name() -> String { "number".to_string() }

    fn is_typeof<Other: Value>(other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_number(other.to_raw()) }
    }
}

impl Value for JsNumber { }

#[repr(C)]
pub struct JsObject(raw::Persistent);

impl Managed for JsObject { }

impl ValueInternal for JsObject {
    fn name() -> String { "object".to_string() }

    fn is_typeof<Other: Value>(other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_object(other.to_raw()) }
    }
}

impl Value for JsObject { }

impl Object for JsObject { }

unsafe impl This for JsObject { }

impl JsObject {
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> &'a JsObject {
        cx.new_infallible(|out, isolate| unsafe {
            neon_runtime::object::new(out, isolate)
        })
    }
}

/// A JavaScript array object, i.e. a value for which `Array.isArray`
/// would return `true`.
#[repr(C)]
pub struct JsArray(raw::Persistent);

impl JsArray {

    pub fn new<'a, C: Context<'a>>(cx: &mut C, len: u32) -> &'a JsArray {
        cx.new_infallible(|out, isolate| unsafe {
            neon_runtime::array::new(out, isolate, len)
        })
    }

    pub fn to_vec<'a, C: Context<'a>>(&self, cx: &mut C) -> NeonResult<Vec<&'a JsValue>> {
        let mut result = Vec::with_capacity(self.len() as usize);
        let mut i = 0;
        loop {
            // Since getting a property can trigger arbitrary code,
            // we have to re-check the length on every iteration.
            if i >= self.len() {
                return Ok(result);
            }
            result.push(self.get(cx, i)?);
            i += 1;
        }
    }

    pub fn len(&self) -> u32 {
        unsafe {
            neon_runtime::array::len(self.to_raw())
        }
    }

}

impl Value for JsArray { }

impl Managed for JsArray { }

impl ValueInternal for JsArray {
    fn name() -> String { "Array".to_string() }

    fn is_typeof<Other: Value>(other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_array(other.to_raw()) }
    }
}

impl Object for JsArray { }

/// A JavaScript function object.
#[repr(C)]
pub struct JsFunction<T: Object=JsObject> {
    raw: raw::Persistent,
    marker: PhantomData<T>,
}

impl<T: Object> Object for JsFunction<T> { }

impl JsFunction {
    pub fn new<'a, C, U>(cx: &mut C, f: fn(FunctionContext) -> NeonResult<&U>) -> NeonResult<&'a JsFunction>
        where C: Context<'a>,
              U: Value
    {
        cx.new(|out, isolate| unsafe {
            let isolate: *mut c_void = std::mem::transmute(isolate);
            let callback = FunctionCallback(f).into_c_callback();
            neon_runtime::fun::new(out, isolate, callback)
        })
    }
}

unsafe fn prepare_args<'a, 'b, C: Context<'a>, A>(cx: &mut C, args: &mut [&'b A]) -> NeonResult<(i32, *mut c_void)>
    where A: Value + 'b
{
    let argv = args.as_mut_ptr();
    let argc = args.len();
    if argc > V8_ARGC_LIMIT {
        return cx.throw_range_error("too many arguments");
    }
    Ok((argc as i32, argv as *mut c_void))
}

impl<CL: Object> JsFunction<CL> {
    pub fn call<'a, 'b, C: Context<'a>, T, A, AS>(&self, cx: &mut C, this: &'b T, args: AS) -> NeonResult<&'a JsValue>
        where T: Value,
              A: Value + 'b,
              AS: IntoIterator<Item=&'b A>
    {
        let mut args = args.into_iter().collect::<Vec<_>>();
        let (argc, argv) = unsafe { prepare_args(cx, &mut args) }?;
        cx.new(|out, isolate| unsafe {
            neon_runtime::fun::call(out, isolate, self.to_raw(), this.to_raw(), argc, argv)
        })
    }

    pub fn construct<'a, 'b, C: Context<'a>, A, AS>(&self, cx: &mut C, args: AS) -> NeonResult<&'a CL>
        where A: Value + 'b,
              AS: IntoIterator<Item=&'b A>
    {
        let mut args = args.into_iter().collect::<Vec<_>>();
        let (argc, argv) = unsafe { prepare_args(cx, &mut args) }?;
        cx.new(|out, isolate| unsafe {
            neon_runtime::fun::construct(out, isolate, self.to_raw(), argc, argv)
        })
    }
}

impl<T: Object> Managed for JsFunction<T> { }

impl<T: Object> ValueInternal for JsFunction<T> {
    fn name() -> String { "function".to_string() }

    fn is_typeof<Other: Value>(other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_function(other.to_raw()) }
    }
}

impl<T: Object> Value for JsFunction<T> { }

// Maximum number of function arguments in V8.
const V8_ARGC_LIMIT: usize = 65535;
