pub mod binary;
pub mod error;

use std::mem;
use std::os::raw::c_void;
use neon_sys;
use neon_sys::raw;
use neon_sys::tag::Tag;
use internal::mem::{Handle, HandleInternal};
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::vm::{VmResult, Throw, JsResult, Isolate, CallbackInfo, Call, exec_function_body};
use internal::js::error::JsTypeError;

pub trait ValueInternal: Copy {
    fn to_raw(self) -> raw::Local;

    fn from_raw(h: raw::Local) -> Self;

    fn is_typeof<Other: Value>(other: Other) -> bool;

    fn downcast<Other: Value>(other: Other) -> Option<Self> {
        if Self::is_typeof(other) {
            Some(Self::from_raw(other.to_raw()))
        } else {
            None
        }
    }

    fn cast<'a, T: Value, F: FnOnce(raw::Local) -> T>(self, f: F) -> Handle<'a, T> {
        Handle::new(f(self.to_raw()))
    }
}

pub fn build<'a, T: Value, F: FnOnce(&mut raw::Local) -> bool>(init: F) -> JsResult<'a, T> {
    unsafe {
        let mut local: raw::Local = mem::zeroed();
        if init(&mut local) {
            Ok(Handle::new(T::from_raw(local)))
        } else {
            Err(Throw)
        }
    }
}

pub trait SuperType<T: Value> {
    fn upcast_internal(T) -> Self;
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
    fn to_string<'a, T: Scope<'a>>(self, _: &mut T) -> JsResult<'a, JsString> {
        build(|out| { unsafe { neon_sys::convert::to_string(out, self.to_raw()) } })
    }

    fn as_value<'a, T: Scope<'a>>(self, _: &mut T) -> Handle<'a, JsValue> {
        JsValue::new_internal(self.to_raw())
    }
}

/// A wrapper type for JavaScript values that makes it convenient to
/// check a value's type dynamically using Rust's pattern-matching.
pub enum Variant<'a> {
    Null(Handle<'a, JsNull>),
    Undefined(Handle<'a, JsUndefined>),
    Boolean(Handle<'a, JsBoolean>),
    Integer(Handle<'a, JsInteger>),
    Number(Handle<'a, JsNumber>),
    String(Handle<'a, JsString>),
    Object(Handle<'a, JsObject>),
    Array(Handle<'a, JsArray>),
    Function(Handle<'a, JsFunction>),
    Other(Handle<'a, JsValue>)
}


/// A JavaScript value of any type.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsValue(raw::Local);

impl Value for JsValue { }

impl ValueInternal for JsValue {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsValue(h) }

    fn is_typeof<Other: Value>(_: Other) -> bool {
        true
    }
}

impl<'a> Handle<'a, JsValue> {
    pub fn variant(self) -> Variant<'a> {
        match unsafe { neon_sys::tag::of(self.to_raw()) } {
            Tag::Null => Variant::Null(JsNull::new()),
            Tag::Undefined => Variant::Undefined(JsUndefined::new()),
            Tag::Boolean => Variant::Boolean(Handle::new(JsBoolean(self.to_raw()))),
            Tag::Integer => Variant::Integer(Handle::new(JsInteger(self.to_raw()))),
            Tag::Number => Variant::Number(Handle::new(JsNumber(self.to_raw()))),
            Tag::String => Variant::String(Handle::new(JsString(self.to_raw()))),
            Tag::Object => Variant::Object(Handle::new(JsObject(self.to_raw()))),
            Tag::Array => Variant::Array(Handle::new(JsArray(self.to_raw()))),
            Tag::Function => Variant::Function(Handle::new(JsFunction(self.to_raw()))),
            Tag::Other => Variant::Other(self.clone())
        }
    }
}

pub trait JsValueInternal {
    fn new_internal<'a>(value: raw::Local) -> Handle<'a, JsValue>;
}

impl JsValueInternal for JsValue {
    fn new_internal<'a>(value: raw::Local) -> Handle<'a, JsValue> {
        Handle::new(JsValue(value))
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
}

impl Value for JsUndefined { }

impl ValueInternal for JsUndefined {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsUndefined(h) }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_undefined(other.to_raw()) }
    }
}

pub trait JsUndefinedInternal {
    fn new_internal<'a>() -> Handle<'a, JsUndefined>;
}

impl JsUndefinedInternal for JsUndefined {
    fn new_internal<'a>() -> Handle<'a, JsUndefined> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::primitive::undefined(&mut local);
            Handle::new(JsUndefined(local))
        }
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
}

impl Value for JsNull { }

impl ValueInternal for JsNull {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsNull(h) }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_null(other.to_raw()) }
    }
}

pub trait JsNullInternal {
    fn new_internal<'a>() -> Handle<'a, JsNull>;
}

impl JsNullInternal for JsNull {
    fn new_internal<'a>() -> Handle<'a, JsNull> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::primitive::null(&mut local);
            Handle::new(JsNull(local))
        }
    }
}

/// A JavaScript boolean primitive value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsBoolean(raw::Local);

impl JsBoolean {
    pub fn new<'a, T: Scope<'a>>(_: &mut T, b: bool) -> Handle<'a, JsBoolean> {
        JsBoolean::new_internal(b)
    }
}

impl Value for JsBoolean { }

impl ValueInternal for JsBoolean {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsBoolean(h) }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_boolean(other.to_raw()) }
    }
}

pub trait JsBooleanInternal {
    fn new_internal<'a>(b: bool) -> Handle<'a, JsBoolean>;
}

impl JsBooleanInternal for JsBoolean {
    fn new_internal<'a>(b: bool) -> Handle<'a, JsBoolean> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::primitive::boolean(&mut local, b);
            Handle::new(JsBoolean(local))
        }
    }
}

impl JsBoolean {
    pub fn value(self) -> bool {
        unsafe {
            neon_sys::primitive::boolean_value(self.to_raw())
        }
    }
}

/// A JavaScript string primitive value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsString(raw::Local);

impl Value for JsString { }

impl ValueInternal for JsString {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsString(h) }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_string(other.to_raw()) }
    }
}

impl JsString {
    pub fn size(self) -> isize {
        unsafe {
            neon_sys::string::utf8_len(self.to_raw())
        }
    }

    pub fn value(self) -> String {
        unsafe {
            // FIXME: use StringBytes::StorageSize instead?
            // FIXME: audit all these isize -> usize casts
            let capacity = neon_sys::string::utf8_len(self.to_raw());
            let mut buffer: Vec<u8> = Vec::with_capacity(capacity as usize);
            let p = buffer.as_mut_ptr();
            mem::forget(buffer);
            let len = neon_sys::string::data(p, capacity, self.to_raw());
            String::from_raw_parts(p, len as usize, capacity as usize)
        }
    }

    pub fn new<'a, T: Scope<'a>>(scope: &mut T, val: &str) -> Option<Handle<'a, JsString>> {
        JsString::new_internal(scope.isolate(), val)
    }

    pub fn new_or_throw<'a, T: Scope<'a>>(scope: &mut T, val: &str) -> VmResult<Handle<'a, JsString>> {
        match JsString::new(scope, val) {
            Some(v) => Ok(v),
            // FIXME: should this be a different error type?
            None => JsTypeError::throw("invalid string contents")
        }
    }
}

pub trait JsStringInternal {
    fn new_internal<'a>(isolate: *mut Isolate, val: &str) -> Option<Handle<'a, JsString>>;
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

fn lower_str_unwrap(s: &str) -> (*const u8, i32) {
    lower_str(s).unwrap_or_else(|| {
        panic!("{} < i32::MAX", s.len())
    })
}

impl JsStringInternal for JsString {
    fn new_internal<'a>(isolate: *mut Isolate, val: &str) -> Option<Handle<'a, JsString>> {
        let (ptr, len) = match lower_str(val) {
            Some(pair) => pair,
            None => { return None; }
        };
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            if neon_sys::string::new(&mut local, mem::transmute(isolate), ptr, len) {
                Some(Handle::new(JsString(local)))
            } else {
                None
            }
        }
    }
}


/// A JavaScript number value whose value is known statically to be a
/// 32-bit integer.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsInteger(raw::Local);

impl JsInteger {
    pub fn new<'a, T: Scope<'a>>(scope: &mut T, i: i32) -> Handle<'a, JsInteger> {
        JsInteger::new_internal(scope.isolate(), i)
    }
}

impl Value for JsInteger { }

impl ValueInternal for JsInteger {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsInteger(h) }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_integer(other.to_raw()) }
    }
}

pub trait JsIntegerInternal {
    fn new_internal<'a>(isolate: *mut Isolate, i: i32) -> Handle<'a, JsInteger>;
}

impl JsIntegerInternal for JsInteger {
    fn new_internal<'a>(isolate: *mut Isolate, i: i32) -> Handle<'a, JsInteger> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::primitive::integer(&mut local, mem::transmute(isolate), i);
            Handle::new(JsInteger(local))
        }
    }
}

impl JsInteger {
    pub fn is_u32(self) -> bool {
        unsafe {
            neon_sys::primitive::is_u32(self.to_raw())
        }
    }

    pub fn is_i32(self) -> bool {
        unsafe {
            neon_sys::primitive::is_i32(self.to_raw())
        }
    }

    pub fn value(self) -> i64 {
        unsafe {
            neon_sys::primitive::integer_value(self.to_raw())
        }
    }
}

/// A JavaScript number value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsNumber(raw::Local);

impl JsNumber {
    pub fn new<'a, T: Scope<'a>>(scope: &mut T, v: f64) -> Handle<'a, JsNumber> {
        JsNumber::new_internal(scope.isolate(), v)
    }
}

impl Value for JsNumber { }

impl ValueInternal for JsNumber {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsNumber(h) }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_number(other.to_raw()) }
    }
}

pub trait JsNumberInternal {
    fn new_internal<'a>(isolate: *mut Isolate, v: f64) -> Handle<'a, JsNumber>;
}

impl JsNumberInternal for JsNumber {
    fn new_internal<'a>(isolate: *mut Isolate, v: f64) -> Handle<'a, JsNumber> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::primitive::number(&mut local, mem::transmute(isolate), v);
            Handle::new(JsNumber(local))
        }
    }
}

impl JsNumber {
    pub fn value(self) -> f64 {
        unsafe {
            neon_sys::primitive::number_value(self.to_raw())
        }
    }
}

/// A JavaScript object.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsObject(raw::Local);

impl Value for JsObject { }

impl ValueInternal for JsObject {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsObject(h) }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_object(other.to_raw()) }
    }
}

trait PropertyName {
    unsafe fn get(self, out: &mut raw::Local, obj: raw::Local) -> bool;
    unsafe fn set(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool;
}

impl PropertyName for u32 {
    unsafe fn get(self, out: &mut raw::Local, obj: raw::Local) -> bool {
        neon_sys::object::get_index(out, obj, self)
    }

    unsafe fn set(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
        neon_sys::object::set_index(out, obj, self, val)
    }
}

impl<'a, K: Value> PropertyName for Handle<'a, K> {
    unsafe fn get(self, out: &mut raw::Local, obj: raw::Local) -> bool {
        neon_sys::object::get(out, obj, self.to_raw())
    }

    unsafe fn set(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
        neon_sys::object::set(out, obj, self.to_raw(), val)
    }
}

impl<'a> PropertyName for &'a str {
    unsafe fn get(self, out: &mut raw::Local, obj: raw::Local) -> bool {
        let (ptr, len) = lower_str_unwrap(self);
        neon_sys::object::get_string(out, obj, ptr, len)
    }

    unsafe fn set(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
        let (ptr, len) = lower_str_unwrap(self);
        neon_sys::object::set_string(out, obj, ptr, len, val)
    }
}

/// The trait of all object types.
pub trait Object: Value {
    fn get<'a, T: Scope<'a>, K: PropertyName>(self, _: &mut T, key: K) -> VmResult<Handle<'a, JsValue>> {
        build(|out| { unsafe { key.get(out, self.to_raw()) } })
    }

    fn get_own_property_names<'a, T: Scope<'a>>(self, _: &mut T) -> JsResult<'a, JsArray> {
        build(|out| { unsafe { neon_sys::object::get_own_property_names(out, self.to_raw()) } })
    }

    fn set<K: PropertyName, V: Value>(self, key: K, val: Handle<V>) -> VmResult<bool> {
        let mut result = false;
        if unsafe { key.set(&mut result, self.to_raw(), val.to_raw()) } {
            Ok(result)
        } else {
            Err(Throw)
        }
    }
}

impl Object for JsObject { }

pub trait JsObjectInternal {
    fn new_internal<'a>() -> Handle<'a, JsObject>;
    fn build<'a, F: FnOnce(&mut raw::Local)>(init: F) -> Handle<'a, JsObject>;
}

impl JsObjectInternal for JsObject {
    fn new_internal<'a>() -> Handle<'a, JsObject> {
        JsObject::build(|out| { unsafe { neon_sys::object::new(out) } })
    }

    fn build<'a, F: FnOnce(&mut raw::Local)>(init: F) -> Handle<'a, JsObject> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            init(&mut local);
            Handle::new(JsObject(local))
        }
    }
}


impl JsObject {
    pub fn new<'a, T: Scope<'a>>(_: &mut T) -> Handle<'a, JsObject> {
        JsObject::new_internal()
    }
}

/// A JavaScript array object, i.e. a value for which `Array.isArray`
/// would return `true`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsArray(raw::Local);

impl JsArray {
    pub fn new<'a, T: Scope<'a>>(scope: &mut T, len: u32) -> Handle<'a, JsArray> {
        JsArray::new_internal(scope.isolate(), len)
    }
}

impl Value for JsArray { }

impl ValueInternal for JsArray {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsArray(h) }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_array(other.to_raw()) }
    }
}

pub trait JsArrayInternal {
    fn new_internal<'a>(isolate: *mut Isolate, len: u32) -> Handle<'a, JsArray>;
}

impl JsArrayInternal for JsArray {
    fn new_internal<'a>(isolate: *mut Isolate, len: u32) -> Handle<'a, JsArray> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::array::new(&mut local, mem::transmute(isolate), len);
            Handle::new(JsArray(local))
        }
    }
}

impl JsArray {
    pub fn to_vec<'a, T: Scope<'a>>(self, scope: &mut T) -> VmResult<Vec<Handle<'a, JsValue>>> {
        let mut result = Vec::with_capacity(self.len() as usize);
        let mut i = 0;
        loop {
            // Since getting a property can trigger arbitrary code,
            // we have to re-check the length on every iteration.
            if i >= self.len() {
                return Ok(result);
            }
            result.push(try!(self.get(scope, i)));
            i += 1;
        }
    }

    pub fn len(self) -> u32 {
        unsafe {
            neon_sys::array::len(self.to_raw())
        }
    }
}

impl Object for JsArray { }

/// A JavaScript function object.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsFunction(raw::Local);

impl JsFunction {
    pub fn new<'a, T: Scope<'a>, U: Value>(scope: &mut T, f: fn(Call) -> JsResult<U>) -> JsResult<'a, JsFunction> {
        build(|out| {
            unsafe {
                let isolate: *mut c_void = mem::transmute(scope.isolate());
                let callback: extern "C" fn(&CallbackInfo) = invoke_nanny_function::<U>;
                let callback: *mut c_void = mem::transmute(callback);
                let kernel: *mut c_void = mem::transmute(f);
                neon_sys::fun::new(out, isolate, callback, kernel)
            }
        })
    }
}

extern "C" fn invoke_nanny_function<U: Value>(info: &CallbackInfo) {
    let mut scope = RootScope::new(unsafe { mem::transmute(neon_sys::call::get_isolate(mem::transmute(info))) });
    exec_function_body(info, &mut scope, |call| {
        let data = info.data();
        let kernel: fn(Call) -> JsResult<U> = unsafe { mem::transmute(neon_sys::fun::get_kernel(data.to_raw())) };
        if let Ok(value) = kernel(call) {
            info.set_return(value);
        }
    });
}

impl Value for JsFunction { }

impl ValueInternal for JsFunction {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsFunction(h) }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_function(other.to_raw()) }
    }
}
