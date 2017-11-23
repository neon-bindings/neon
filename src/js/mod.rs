//! Types and traits representing JavaScript values.

pub mod binary;
pub mod error;
pub mod class;

use std::mem;
use std::os::raw::c_void;
use std::marker::PhantomData;
use neon_runtime;
use neon_runtime::raw;
use neon_runtime::tag::Tag;
use mem::{Handle, Managed};
use scope::Scope;
use vm::{VmResult, Throw, JsResult, Call, This, Kernel};
use vm::internal::Isolate;
use js::error::{JsError, Kind};
use self::internal::{ValueInternal, SuperType, FunctionKernel};

pub(crate) mod internal {
    use std::mem;
    use std::os::raw::c_void;
    use neon_runtime;
    use neon_runtime::raw;
    use mem::{Handle, Managed};
    use vm::{JsResult, CallbackInfo, Call, Kernel};
    use js::error::convert_panics;
    use super::Value;

    pub trait ValueInternal: Managed {
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

    pub trait SuperType<T: Value> {
        fn upcast_internal(T) -> Self;
    }

    #[repr(C)]
    pub struct FunctionKernel<T: Value>(pub fn(Call) -> JsResult<T>);

    impl<T: Value> Kernel<()> for FunctionKernel<T> {
        extern "C" fn callback(info: &CallbackInfo) {
            info.scope().with(|scope| {
                let data = info.data();
                let FunctionKernel(kernel) = unsafe { Self::from_wrapper(data.to_raw()) };
                let call = info.as_call(scope);
                if let Ok(value) = convert_panics(|| { kernel(call) }) {
                    info.set_return(value);
                }
            })
        }

        unsafe fn from_wrapper(h: raw::Local) -> Self {
            FunctionKernel(mem::transmute(neon_runtime::fun::get_kernel(h)))
        }

        fn as_ptr(self) -> *mut c_void {
            unsafe { mem::transmute(self.0) }
        }
    }
}

pub(crate) fn build<'a, T: Managed, F: FnOnce(&mut raw::Local) -> bool>(init: F) -> JsResult<'a, T> {
    unsafe {
        let mut local: raw::Local = mem::zeroed();
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
    fn to_string<'a, T: Scope<'a>>(self, _: &mut T) -> JsResult<'a, JsString> {
        build(|out| { unsafe { neon_runtime::convert::to_string(out, self.to_raw()) } })
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
    // DEPRECATE(0.2)
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

impl Managed for JsValue {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsValue(h) }
}

impl ValueInternal for JsValue {
    fn is_typeof<Other: Value>(_: Other) -> bool {
        true
    }
}

unsafe impl This for JsValue {
    fn as_this(h: raw::Local) -> Self {
        JsValue(h)
    }
}

impl<'a> Handle<'a, JsValue> {
    pub fn variant(self) -> Variant<'a> {
        match unsafe { neon_runtime::tag::of(self.to_raw()) } {
            Tag::Null => Variant::Null(JsNull::new()),
            Tag::Undefined => Variant::Undefined(JsUndefined::new()),
            Tag::Boolean => Variant::Boolean(Handle::new_internal(JsBoolean(self.to_raw()))),
            // DEPRECATE(0.2)
            Tag::Integer => Variant::Integer(Handle::new_internal(JsInteger(self.to_raw()))),
            Tag::Number => Variant::Number(Handle::new_internal(JsNumber(self.to_raw()))),
            Tag::String => Variant::String(Handle::new_internal(JsString(self.to_raw()))),
            Tag::Object => Variant::Object(Handle::new_internal(JsObject(self.to_raw()))),
            Tag::Array => Variant::Array(Handle::new_internal(JsArray(self.to_raw()))),
            Tag::Function => Variant::Function(Handle::new_internal(JsFunction { raw: self.to_raw(), marker: PhantomData })),
            Tag::Other => Variant::Other(self.clone())
        }
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
            let mut local: raw::Local = mem::zeroed();
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
            let mut local: raw::Local = mem::zeroed();
            neon_runtime::primitive::undefined(&mut local);
            JsUndefined(local)
        }
    }
}

impl ValueInternal for JsUndefined {
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
            let mut local: raw::Local = mem::zeroed();
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
    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_null(other.to_raw()) }
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

    pub(crate) fn new_internal<'a>(b: bool) -> Handle<'a, JsBoolean> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
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
    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_boolean(other.to_raw()) }
    }
}

/// A JavaScript string primitive value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsString(raw::Local);

impl Value for JsString { }

impl Managed for JsString {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsString(h) }
}

impl ValueInternal for JsString {
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
            mem::forget(buffer);
            let len = neon_runtime::string::data(p, capacity, self.to_raw());
            String::from_raw_parts(p, len as usize, capacity as usize)
        }
    }

    pub fn new<'a, T: Scope<'a>>(scope: &mut T, val: &str) -> Option<Handle<'a, JsString>> {
        JsString::new_internal(scope.isolate(), val)
    }

    pub fn new_or_throw<'a, T: Scope<'a>>(scope: &mut T, val: &str) -> VmResult<Handle<'a, JsString>> {
        match JsString::new(scope, val) {
            Some(v) => Ok(v),
            None => JsError::throw(Kind::TypeError, "invalid string contents")
        }
    }

    pub(crate) fn new_internal<'a>(isolate: Isolate, val: &str) -> Option<Handle<'a, JsString>> {
        let (ptr, len) = match lower_str(val) {
            Some(pair) => pair,
            None => { return None; }
        };
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            if neon_runtime::string::new(&mut local, isolate.to_raw(), ptr, len) {
                Some(Handle::new_internal(JsString(local)))
            } else {
                None
            }
        }
    }
}

pub trait ToJsString {
    fn to_js_string<'a, T: Scope<'a>>(&self, scope: &mut T) -> Handle<'a, JsString>;
}

impl<'b> ToJsString for Handle<'b, JsString> {
    fn to_js_string<'a, T: Scope<'a>>(&self, _: &mut T) -> Handle<'a, JsString> {
        Handle::new_internal(JsString::from_raw(self.to_raw()))
    }
}

impl<'b> ToJsString for &'b str {
    fn to_js_string<'a, T: Scope<'a>>(&self, scope: &mut T) -> Handle<'a, JsString> {
        match JsString::new_internal(scope.isolate(), self) {
            Some(s) => s,
            None => JsString::new_internal(scope.isolate(), "").unwrap()
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

fn lower_str_unwrap(s: &str) -> (*const u8, i32) {
    lower_str(s).unwrap_or_else(|| {
        panic!("{} < i32::MAX", s.len())
    })
}

// DEPRECATE(0.2)
/// A JavaScript number value whose value is known statically to be a
/// 32-bit integer.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsInteger(raw::Local);

impl JsInteger {
    pub fn new<'a, T: Scope<'a>>(scope: &mut T, i: i32) -> Handle<'a, JsInteger> {
        JsInteger::new_internal(scope.isolate(), i)
    }

    pub(crate) fn new_internal<'a>(isolate: Isolate, i: i32) -> Handle<'a, JsInteger> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_runtime::primitive::integer(&mut local, isolate.to_raw(), i);
            Handle::new_internal(JsInteger(local))
        }
    }

    pub fn is_u32(self) -> bool {
        unsafe {
            neon_runtime::primitive::is_u32(self.to_raw())
       }
    }
    pub fn is_i32(self) -> bool {
        unsafe {
            neon_runtime::primitive::is_i32(self.to_raw())
        }
    }

    pub fn value(self) -> i64 {
        unsafe {
            neon_runtime::primitive::integer_value(self.to_raw())
        }
    }
}

impl Value for JsInteger { }

impl Managed for JsInteger {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsInteger(h) }
}

impl ValueInternal for JsInteger {
    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_integer(other.to_raw()) }
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

    pub(crate) fn new_internal<'a>(isolate: Isolate, v: f64) -> Handle<'a, JsNumber> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
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
    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_object(other.to_raw()) }
    }
}

/// A property key in a JavaScript object.
pub trait Key {
    unsafe fn get(self, out: &mut raw::Local, obj: raw::Local) -> bool;
    unsafe fn set(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool;
}

impl Key for u32 {
    unsafe fn get(self, out: &mut raw::Local, obj: raw::Local) -> bool {
        neon_runtime::object::get_index(out, obj, self)
    }

    unsafe fn set(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
        neon_runtime::object::set_index(out, obj, self, val)
    }
}

impl<'a, K: Value> Key for Handle<'a, K> {
    unsafe fn get(self, out: &mut raw::Local, obj: raw::Local) -> bool {
        neon_runtime::object::get(out, obj, self.to_raw())
    }

    unsafe fn set(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
        neon_runtime::object::set(out, obj, self.to_raw(), val)
    }
}

impl<'a> Key for &'a str {
    unsafe fn get(self, out: &mut raw::Local, obj: raw::Local) -> bool {
        let (ptr, len) = lower_str_unwrap(self);
        neon_runtime::object::get_string(out, obj, ptr, len)
    }

    unsafe fn set(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
        let (ptr, len) = lower_str_unwrap(self);
        neon_runtime::object::set_string(out, obj, ptr, len, val)
    }
}

/// The trait of all object types.
pub trait Object: Value {
    fn get<'a, T: Scope<'a>, K: Key>(self, _: &mut T, key: K) -> VmResult<Handle<'a, JsValue>> {
        build(|out| { unsafe { key.get(out, self.to_raw()) } })
    }

    fn get_own_property_names<'a, T: Scope<'a>>(self, _: &mut T) -> JsResult<'a, JsArray> {
        build(|out| { unsafe { neon_runtime::object::get_own_property_names(out, self.to_raw()) } })
    }

    fn set<K: Key, V: Value>(self, key: K, val: Handle<V>) -> VmResult<bool> {
        let mut result = false;
        if unsafe { key.set(&mut result, self.to_raw(), val.to_raw()) } {
            Ok(result)
        } else {
            Err(Throw)
        }
    }
}

impl Object for JsObject { }

impl JsObject {
    pub fn new<'a, T: Scope<'a>>(_: &mut T) -> Handle<'a, JsObject> {
        JsObject::new_internal()
    }

    pub(crate) fn new_internal<'a>() -> Handle<'a, JsObject> {
        JsObject::build(|out| { unsafe { neon_runtime::object::new(out) } })
    }

    pub(crate) fn build<'a, F: FnOnce(&mut raw::Local)>(init: F) -> Handle<'a, JsObject> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
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
    pub fn new<'a, T: Scope<'a>>(scope: &mut T, len: u32) -> Handle<'a, JsArray> {
        JsArray::new_internal(scope.isolate(), len)
    }

    pub(crate) fn new_internal<'a>(isolate: Isolate, len: u32) -> Handle<'a, JsArray> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_runtime::array::new(&mut local, isolate.to_raw(), len);
            Handle::new_internal(JsArray(local))
        }
    }

    pub fn to_vec<'a, T: Scope<'a>>(self, scope: &mut T) -> VmResult<Vec<Handle<'a, JsValue>>> {
        let mut result = Vec::with_capacity(self.len() as usize);
        let mut i = 0;
        loop {
            // Since getting a property can trigger arbitrary code,
            // we have to re-check the length on every iteration.
            if i >= self.len() {
                return Ok(result);
            }
            result.push(self.get(scope, i)?);
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

unsafe fn prepare_call<'a, 'b, S: Scope<'a>, A>(scope: &mut S, args: &mut [Handle<'b, A>]) -> VmResult<(*mut c_void, i32, *mut c_void)>
    where A: Value + 'b
{
    let argv = args.as_mut_ptr();
    let argc = args.len();
    if argc > V8_ARGC_LIMIT {
        return JsError::throw(Kind::RangeError, "too many arguments");
    }
    let isolate: *mut c_void = mem::transmute(scope.isolate().to_raw());
    Ok((isolate, argc as i32, argv as *mut c_void))
}

impl JsFunction {
    pub fn new<'a, T: Scope<'a>, U: Value>(scope: &mut T, f: fn(Call) -> JsResult<U>) -> JsResult<'a, JsFunction> {
        build(|out| {
            unsafe {
                let isolate: *mut c_void = mem::transmute(scope.isolate().to_raw());
                let (callback, kernel) = FunctionKernel(f).export();
                neon_runtime::fun::new(out, isolate, callback, kernel)
            }
        })
    }
}

impl<C: Object> JsFunction<C> {
    pub fn call<'a, 'b, S: Scope<'a>, T, A, AS>(self, scope: &mut S, this: Handle<'b, T>, args: AS) -> JsResult<'a, JsValue>
        where T: Value,
              A: Value + 'b,
              AS: IntoIterator<Item=Handle<'b, A>>
    {
        let mut args = args.into_iter().collect::<Vec<_>>();
        let (isolate, argc, argv) = unsafe { prepare_call(scope, &mut args) }?;
        build(|out| {
            unsafe {
                neon_runtime::fun::call(out, isolate, self.to_raw(), this.to_raw(), argc, argv)
            }
        })
    }

    pub fn construct<'a, 'b, S: Scope<'a>, A, AS>(self, scope: &mut S, args: AS) -> JsResult<'a, C>
        where A: Value + 'b,
              AS: IntoIterator<Item=Handle<'b, A>>
    {
        let mut args = args.into_iter().collect::<Vec<_>>();
        let (isolate, argc, argv) = unsafe { prepare_call(scope, &mut args) }?;
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
    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_function(other.to_raw()) }
    }
}
