//! Types and traits for working with JavaScript values.

pub(crate) mod binary;
pub(crate) mod error;
pub(crate) mod mem;

pub(crate) mod internal;

use std;
use std::fmt;
use std::os::raw::c_void;
use std::marker::PhantomData;
use neon_runtime;
use neon_runtime::raw;
use cx::{Context, FunctionContext};
use cx::internal::Isolate;
use result::{NeonResult, Throw, ResultExt};
use object::{Object, This};
use object::class::Callback;
use self::internal::{ValueInternal, SuperType, FunctionCallback};

pub use self::binary::{JsBuffer, JsArrayBuffer, BinaryData, BinaryViewType};
pub use self::error::{JsError, ErrorKind};
pub use self::mem::{Handle, Managed, DowncastError, DowncastResult};

/// The result of a computation that produces a JS value and might send the JS engine into a throwing state.
pub type JsResult<'b, T> = NeonResult<Handle<'b, T>>;

pub(crate) fn build<'a, T: Managed, F: FnOnce(&mut raw::Local) -> bool>(init: F) -> JsResult<'a, T> {
    unsafe {
        let mut local: raw::Local = std::mem::zeroed();
        if init(&mut local) {
            Ok(Handle::new_internal(T::from_raw(local)))
        } else {
            Err(Throw)
        }
    }
}

impl<T: Value> SuperType<T> for JsValue {
    fn upcast_internal(v: T) -> JsValue {
        JsValue(v.to_raw())
    }
}

impl<T: Object> SuperType<T> for JsObject {
    fn upcast_internal(v: T) -> JsObject {
        JsObject(v.to_raw())
    }
}

/// The trait shared by all JavaScript values.
pub trait Value: ValueInternal {
    fn to_string<'a, C: Context<'a>>(self, _: &mut C) -> JsResult<'a, JsString> {
        build(|out| { unsafe { neon_runtime::convert::to_string(out, self.to_raw()) } })
    }

    fn as_value<'a, C: Context<'a>>(self, _: &mut C) -> Handle<'a, JsValue> {
        JsValue::new_internal(self.to_raw())
    }
}

/// A JavaScript value of any type.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsValue(raw::Local);

impl Value for JsValue { }

impl Managed for JsValue {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsValue(h) }
}

impl ValueInternal for JsValue {
    fn name() -> String { "any".to_string() }

    fn is_typeof<Other: Value>(_: Other) -> bool {
        true
    }
}

unsafe impl This for JsValue {
    fn as_this(h: raw::Local) -> Self {
        JsValue(h)
    }
}

impl JsValue {
    pub(crate) fn new_internal<'a>(value: raw::Local) -> Handle<'a, JsValue> {
        Handle::new_internal(JsValue(value))
    }
}

/// The JavaScript `undefined` value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsUndefined(raw::Local);

impl JsUndefined {
    pub fn new<'a>() -> Handle<'a, JsUndefined> {
        JsUndefined::new_internal()
    }

    pub(crate) fn new_internal<'a>() -> Handle<'a, JsUndefined> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::primitive::undefined(&mut local);
            Handle::new_internal(JsUndefined(local))
        }
    }
}

impl Value for JsUndefined { }

impl Managed for JsUndefined {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsUndefined(h) }
}

unsafe impl This for JsUndefined {
    fn as_this(_: raw::Local) -> Self {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::primitive::undefined(&mut local);
            JsUndefined(local)
        }
    }
}

impl ValueInternal for JsUndefined {
    fn name() -> String { "undefined".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_undefined(other.to_raw()) }
    }
}

/// The JavaScript `null` value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsNull(raw::Local);

impl JsNull {
    pub fn new<'a>() -> Handle<'a, JsNull> {
        JsNull::new_internal()
    }

    pub(crate) fn new_internal<'a>() -> Handle<'a, JsNull> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::primitive::null(&mut local);
            Handle::new_internal(JsNull(local))
        }
    }
}

impl Value for JsNull { }

impl Managed for JsNull {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsNull(h) }
}

impl ValueInternal for JsNull {
    fn name() -> String { "null".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_null(other.to_raw()) }
    }
}

/// A JavaScript boolean primitive value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsBoolean(raw::Local);

impl JsBoolean {
    pub fn new<'a, C: Context<'a>>(_: &mut C, b: bool) -> Handle<'a, JsBoolean> {
        JsBoolean::new_internal(b)
    }

    pub(crate) fn new_internal<'a>(b: bool) -> Handle<'a, JsBoolean> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::primitive::boolean(&mut local, b);
            Handle::new_internal(JsBoolean(local))
        }
    }

    pub fn value(self) -> bool {
        unsafe {
            neon_runtime::primitive::boolean_value(self.to_raw())
        }
    }
}

impl Value for JsBoolean { }

impl Managed for JsBoolean {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsBoolean(h) }
}

impl ValueInternal for JsBoolean {
    fn name() -> String { "boolean".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_boolean(other.to_raw()) }
    }
}

/// A JavaScript string primitive value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsString(raw::Local);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct StringOverflow(usize);

impl fmt::Display for StringOverflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "string size out of range: {}", self.0)
    }
}

pub type StringResult<'a> = Result<Handle<'a, JsString>, StringOverflow>;

impl<'a> ResultExt<'a, JsString> for StringResult<'a> {
    fn unwrap_or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> JsResult<'a, JsString> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => JsError::throw(cx, ErrorKind::RangeError, &e.to_string())
        }
    }
}

impl Value for JsString { }

impl Managed for JsString {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsString(h) }
}

impl ValueInternal for JsString {
    fn name() -> String { "string".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
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

    pub fn new<'a, C: Context<'a>, S: AsRef<str>>(cx: &mut C, val: S) -> Handle<'a, JsString> {
        JsString::try_new(cx, val).unwrap()
    }

    pub fn try_new<'a, C: Context<'a>, S: AsRef<str>>(cx: &mut C, val: S) -> StringResult<'a> {
        let val = val.as_ref();
        match JsString::new_internal(cx.isolate(), val) {
            Some(s) => Ok(s),
            None => Err(StringOverflow(val.len()))
        }
    }

    pub(crate) fn new_internal<'a>(isolate: Isolate, val: &str) -> Option<Handle<'a, JsString>> {
        let (ptr, len) = match lower_str(val) {
            Some(pair) => pair,
            None => { return None; }
        };
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            if neon_runtime::string::new(&mut local, isolate.to_raw(), ptr, len) {
                Some(Handle::new_internal(JsString(local)))
            } else {
                None
            }
        }
    }
}

/// A trait for invoking the JS `[[ToString]]` conversion protocol.
pub trait ToJsString {
    /// Invoke the JS `[[ToString]]` conversion protocol.
    fn to_js_string<'a, C: Context<'a>>(&self, cx: &mut C) -> Handle<'a, JsString>;
}

impl<'b> ToJsString for Handle<'b, JsString> {
    fn to_js_string<'a, C: Context<'a>>(&self, _: &mut C) -> Handle<'a, JsString> {
        Handle::new_internal(JsString::from_raw(self.to_raw()))
    }
}

impl<'b> ToJsString for &'b str {
    fn to_js_string<'a, C: Context<'a>>(&self, cx: &mut C) -> Handle<'a, JsString> {
        match JsString::new_internal(cx.isolate(), self) {
            Some(s) => s,
            None => JsString::new_internal(cx.isolate(), "").unwrap()
        }
    }
}

// Lower a &str to the types expected by Node: a const *uint8_t buffer and an int32_t length.
fn lower_str(s: &str) -> Option<(*const u8, i32)> {
    // V8 currently refuses to allocate strings longer than `(1 << 20) - 16` bytes,
    // but in case this changes over time, just ensure the buffer isn't longer than
    // the largest positive signed integer, and delegate the tighter bounds checks
    // to V8.
    let len = s.len();
    if len > (::std::i32::MAX as usize) {
        return None;
    }
    Some((s.as_ptr(), len as i32))
}

pub(crate) fn lower_str_unwrap(s: &str) -> (*const u8, i32) {
    lower_str(s).unwrap_or_else(|| {
        panic!("{} < i32::MAX", s.len())
    })
}

/// A JavaScript number value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsNumber(raw::Local);

impl JsNumber {
    pub fn new<'a, C: Context<'a>, T: Into<f64>>(cx: &mut C, x: T) -> Handle<'a, JsNumber> {
        JsNumber::new_internal(cx.isolate(), x.into())
    }

    pub(crate) fn new_internal<'a>(isolate: Isolate, v: f64) -> Handle<'a, JsNumber> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::primitive::number(&mut local, isolate.to_raw(), v);
            Handle::new_internal(JsNumber(local))
        }
    }

    pub fn value(self) -> f64 {
        unsafe {
            neon_runtime::primitive::number_value(self.to_raw())
        }
    }
}

impl Value for JsNumber { }

impl Managed for JsNumber {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsNumber(h) }
}

impl ValueInternal for JsNumber {
    fn name() -> String { "number".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_number(other.to_raw()) }
    }
}

/// A JavaScript object.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsObject(raw::Local);

impl Value for JsObject { }

impl Managed for JsObject {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsObject(h) }
}

unsafe impl This for JsObject {
    fn as_this(h: raw::Local) -> Self { JsObject(h) }
}

impl ValueInternal for JsObject {
    fn name() -> String { "object".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_object(other.to_raw()) }
    }
}

impl Object for JsObject { }

impl JsObject {
    pub fn new<'a, C: Context<'a>>(_: &mut C) -> Handle<'a, JsObject> {
        JsObject::new_internal()
    }

    pub(crate) fn new_internal<'a>() -> Handle<'a, JsObject> {
        JsObject::build(|out| { unsafe { neon_runtime::object::new(out) } })
    }

    pub(crate) fn build<'a, F: FnOnce(&mut raw::Local)>(init: F) -> Handle<'a, JsObject> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            init(&mut local);
            Handle::new_internal(JsObject(local))
        }
    }
}

/// A JavaScript array object, i.e. a value for which `Array.isArray`
/// would return `true`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsArray(raw::Local);

impl JsArray {
    pub fn new<'a, C: Context<'a>>(cx: &mut C, len: u32) -> Handle<'a, JsArray> {
        JsArray::new_internal(cx.isolate(), len)
    }

    pub(crate) fn new_internal<'a>(isolate: Isolate, len: u32) -> Handle<'a, JsArray> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::array::new(&mut local, isolate.to_raw(), len);
            Handle::new_internal(JsArray(local))
        }
    }

    pub fn to_vec<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<Vec<Handle<'a, JsValue>>> {
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

    pub fn len(self) -> u32 {
        unsafe {
            neon_runtime::array::len(self.to_raw())
        }
    }
}

impl Value for JsArray { }

impl Managed for JsArray {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsArray(h) }
}

impl ValueInternal for JsArray {
    fn name() -> String { "Array".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_array(other.to_raw()) }
    }
}

impl Object for JsArray { }

/// A JavaScript function object.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsFunction<T: Object=JsObject> {
    raw: raw::Local,
    marker: PhantomData<T>
}

impl<T: Object> Object for JsFunction<T> { }

// Maximum number of function arguments in V8.
const V8_ARGC_LIMIT: usize = 65535;

unsafe fn prepare_call<'a, 'b, C: Context<'a>, A>(cx: &mut C, args: &mut [Handle<'b, A>]) -> NeonResult<(*mut c_void, i32, *mut c_void)>
    where A: Value + 'b
{
    let argv = args.as_mut_ptr();
    let argc = args.len();
    if argc > V8_ARGC_LIMIT {
        return JsError::throw(cx, ErrorKind::RangeError, "too many arguments");
    }
    let isolate: *mut c_void = std::mem::transmute(cx.isolate().to_raw());
    Ok((isolate, argc as i32, argv as *mut c_void))
}

impl JsFunction {
    pub fn new<'a, C, U>(cx: &mut C, f: fn(FunctionContext) -> JsResult<U>) -> JsResult<'a, JsFunction>
        where C: Context<'a>,
              U: Value
    {
        build(|out| {
            unsafe {
                let isolate: *mut c_void = std::mem::transmute(cx.isolate().to_raw());
                let callback = FunctionCallback(f).into_c_callback();
                neon_runtime::fun::new(out, isolate, callback)
            }
        })
    }
}

impl<CL: Object> JsFunction<CL> {
    pub fn call<'a, 'b, C: Context<'a>, T, A, AS>(self, cx: &mut C, this: Handle<'b, T>, args: AS) -> JsResult<'a, JsValue>
        where T: Value,
              A: Value + 'b,
              AS: IntoIterator<Item=Handle<'b, A>>
    {
        let mut args = args.into_iter().collect::<Vec<_>>();
        let (isolate, argc, argv) = unsafe { prepare_call(cx, &mut args) }?;
        build(|out| {
            unsafe {
                neon_runtime::fun::call(out, isolate, self.to_raw(), this.to_raw(), argc, argv)
            }
        })
    }

    pub fn construct<'a, 'b, C: Context<'a>, A, AS>(self, cx: &mut C, args: AS) -> JsResult<'a, CL>
        where A: Value + 'b,
              AS: IntoIterator<Item=Handle<'b, A>>
    {
        let mut args = args.into_iter().collect::<Vec<_>>();
        let (isolate, argc, argv) = unsafe { prepare_call(cx, &mut args) }?;
        build(|out| {
            unsafe {
                neon_runtime::fun::construct(out, isolate, self.to_raw(), argc, argv)
            }
        })
    }
}

impl<T: Object> Value for JsFunction<T> { }

impl<T: Object> Managed for JsFunction<T> {
    fn to_raw(self) -> raw::Local { self.raw }

    fn from_raw(h: raw::Local) -> Self {
        JsFunction {
            raw: h,
            marker: PhantomData
        }
    }
}

impl<T: Object> ValueInternal for JsFunction<T> {
    fn name() -> String { "function".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_function(other.to_raw()) }
    }
}
