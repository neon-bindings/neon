use std;
use std::mem;
use std::os::raw::c_void;
use neon_sys;
use neon_sys::raw;
use neon_sys::tag::Tag;
use internal::mem::{Handle, HandleInternal};
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::vm::{Result, Throw, JS, Isolate, CallbackInfo, Call, exec_function_body};
use internal::error::TypeError;

pub trait AnyInternal: Copy {
    fn to_raw(self) -> raw::Local;

    fn from_raw(h: raw::Local) -> Self;

    fn is_typeof<Other: Any>(other: Other) -> bool;

    fn downcast<Other: Any>(other: Other) -> Option<Self> {
        if Self::is_typeof(other) {
            Some(Self::from_raw(other.to_raw()))
        } else {
            None
        }
    }

    fn cast<'a, T: Any, F: FnOnce(raw::Local) -> T>(self, f: F) -> Handle<'a, T> {
        Handle::new(f(self.to_raw()))
    }
}

pub fn build<'a, T: Any, F: FnOnce(&mut raw::Local) -> bool>(init: F) -> JS<'a, T> {
    unsafe {
        let mut local: raw::Local = mem::zeroed();
        if init(&mut local) {
            Ok(Handle::new(T::from_raw(local)))
        } else {
            Err(Throw)
        }
    }
}

pub trait SuperType<T: Any> {
    fn upcast_internal(T) -> Self;
}

impl<T: Any> SuperType<T> for Value {
    fn upcast_internal(v: T) -> Value {
        Value(v.to_raw())
    }
}

impl<T: Object> SuperType<T> for SomeObject {
    fn upcast_internal(v: T) -> SomeObject {
        SomeObject(v.to_raw())
    }
}

pub trait Any: AnyInternal {
    fn to_string<'a, T: Scope<'a>>(self, _: &mut T) -> JS<'a, String> {
        build(|out| { unsafe { neon_sys::convert::to_string(out, self.to_raw()) } })
    }

    fn value<'a, T: Scope<'a>>(self, _: &mut T) -> Handle<'a, Value> {
        Value::new_internal(self.to_raw())
    }
}

pub enum Variant<'a> {
    Null(Handle<'a, Null>),
    Undefined(Handle<'a, Undefined>),
    Boolean(Handle<'a, Boolean>),
    Integer(Handle<'a, Integer>),
    Number(Handle<'a, Number>),
    String(Handle<'a, String>),
    Object(Handle<'a, SomeObject>),
    Array(Handle<'a, Array>),
    Function(Handle<'a, Function>),
    Other(Handle<'a, Value>)
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Value(raw::Local);

impl Any for Value { }

impl AnyInternal for Value {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { Value(h) }

    fn is_typeof<Other: Any>(_: Other) -> bool {
        true
    }
}

impl<'a> Handle<'a, Value> {
    pub fn variant(self) -> Variant<'a> {
        match unsafe { neon_sys::tag::of(self.to_raw()) } {
            Tag::Null => Variant::Null(Null::new()),
            Tag::Undefined => Variant::Undefined(Undefined::new()),
            Tag::Boolean => Variant::Boolean(Handle::new(Boolean(self.to_raw()))),
            Tag::Integer => Variant::Integer(Handle::new(Integer(self.to_raw()))),
            Tag::Number => Variant::Number(Handle::new(Number(self.to_raw()))),
            Tag::String => Variant::String(Handle::new(String(self.to_raw()))),
            Tag::Object => Variant::Object(Handle::new(SomeObject(self.to_raw()))),
            Tag::Array => Variant::Array(Handle::new(Array(self.to_raw()))),
            Tag::Function => Variant::Function(Handle::new(Function(self.to_raw()))),
            Tag::Other => Variant::Other(self.clone())
        }
    }
}

pub trait ValueInternal {
    fn new_internal<'a>(value: raw::Local) -> Handle<'a, Value>;
}

impl ValueInternal for Value {
    fn new_internal<'a>(value: raw::Local) -> Handle<'a, Value> {
        Handle::new(Value(value))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Undefined(raw::Local);

impl Undefined {
    pub fn new<'a>() -> Handle<'a, Undefined> {
        Undefined::new_internal()
    }
}

impl Any for Undefined { }

impl AnyInternal for Undefined {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { Undefined(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_undefined(other.to_raw()) }
    }
}

pub trait UndefinedInternal {
    fn new_internal<'a>() -> Handle<'a, Undefined>;
}

impl UndefinedInternal for Undefined {
    fn new_internal<'a>() -> Handle<'a, Undefined> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::primitive::undefined(&mut local);
            Handle::new(Undefined(local))
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Null(raw::Local);

impl Null {
    pub fn new<'a>() -> Handle<'a, Null> {
        Null::new_internal()
    }
}

impl Any for Null { }

impl AnyInternal for Null {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { Null(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_null(other.to_raw()) }
    }
}

pub trait NullInternal {
    fn new_internal<'a>() -> Handle<'a, Null>;
}

impl NullInternal for Null {
    fn new_internal<'a>() -> Handle<'a, Null> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::primitive::null(&mut local);
            Handle::new(Null(local))
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Boolean(raw::Local);

impl Boolean {
    pub fn new<'a, T: Scope<'a>>(_: &mut T, b: bool) -> Handle<'a, Boolean> {
        Boolean::new_internal(b)
    }
}

impl Any for Boolean { }

impl AnyInternal for Boolean {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { Boolean(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_boolean(other.to_raw()) }
    }
}

pub trait BooleanInternal {
    fn new_internal<'a>(b: bool) -> Handle<'a, Boolean>;
}

impl BooleanInternal for Boolean {
    fn new_internal<'a>(b: bool) -> Handle<'a, Boolean> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::primitive::boolean(&mut local, b);
            Handle::new(Boolean(local))
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct String(raw::Local);

impl Any for String { }

impl AnyInternal for String {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { String(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_string(other.to_raw()) }
    }
}

impl String {
    pub fn size(self) -> isize {
        unsafe {
            neon_sys::string::utf8_len(self.to_raw())
        }
    }

    pub fn data(self) -> std::string::String {
        unsafe {
            // FIXME: use StringBytes::StorageSize instead?
            // FIXME: audit all these isize -> usize casts
            let capacity = neon_sys::string::utf8_len(self.to_raw());
            let mut buffer: Vec<u8> = Vec::with_capacity(capacity as usize);
            let p = buffer.as_mut_ptr();
            mem::forget(buffer);
            let len = neon_sys::string::data(p, capacity, self.to_raw());
            std::string::String::from_raw_parts(p, len as usize, capacity as usize)
        }
    }

    pub fn new<'a, T: Scope<'a>>(scope: &mut T, val: &str) -> Option<Handle<'a, String>> {
        String::new_internal(scope.isolate(), val)
    }

    pub fn new_or_throw<'a, T: Scope<'a>>(scope: &mut T, val: &str) -> Result<Handle<'a, String>> {
        match String::new(scope, val) {
            Some(v) => Ok(v),
            // FIXME: should this be a different error type?
            None => TypeError::throw("invalid string contents")
        }
    }
}

pub trait StringInternal {
    fn new_internal<'a>(isolate: *mut Isolate, val: &str) -> Option<Handle<'a, String>>;
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

impl StringInternal for String {
    fn new_internal<'a>(isolate: *mut Isolate, val: &str) -> Option<Handle<'a, String>> {
        let (ptr, len) = match lower_str(val) {
            Some(pair) => pair,
            None => { return None; }
        };
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            if neon_sys::string::new(&mut local, mem::transmute(isolate), ptr, len) {
                Some(Handle::new(String(local)))
            } else {
                None
            }
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Integer(raw::Local);

impl Integer {
    pub fn new<'a, T: Scope<'a>>(scope: &mut T, i: i32) -> Handle<'a, Integer> {
        Integer::new_internal(scope.isolate(), i)
    }
}

impl Any for Integer { }

impl AnyInternal for Integer {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { Integer(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_integer(other.to_raw()) }
    }
}

pub trait IntegerInternal {
    fn new_internal<'a>(isolate: *mut Isolate, i: i32) -> Handle<'a, Integer>;
}

impl IntegerInternal for Integer {
    fn new_internal<'a>(isolate: *mut Isolate, i: i32) -> Handle<'a, Integer> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::primitive::integer(&mut local, mem::transmute(isolate), i);
            Handle::new(Integer(local))
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Number(raw::Local);

impl Number {
    pub fn new<'a, T: Scope<'a>>(scope: &mut T, v: f64) -> Handle<'a, Number> {
        Number::new_internal(scope.isolate(), v)
    }
}

impl Any for Number { }

impl AnyInternal for Number {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { Number(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_number(other.to_raw()) }
    }
}

pub trait NumberInternal {
    fn new_internal<'a>(isolate: *mut Isolate, v: f64) -> Handle<'a, Number>;
}

impl NumberInternal for Number {
    fn new_internal<'a>(isolate: *mut Isolate, v: f64) -> Handle<'a, Number> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::primitive::number(&mut local, mem::transmute(isolate), v);
            Handle::new(Number(local))
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SomeObject(raw::Local);

impl Any for SomeObject { }

impl AnyInternal for SomeObject {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { SomeObject(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
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

impl<'a, K: Any> PropertyName for Handle<'a, K> {
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

pub trait Object: Any {
    fn get<'a, T: Scope<'a>, K: PropertyName>(self, _: &mut T, key: K) -> Result<Handle<'a, Value>> {
        build(|out| { unsafe { key.get(out, self.to_raw()) } })
    }

    fn get_own_property_names<'a, T: Scope<'a>>(self, _: &mut T) -> JS<'a, Array> {
        build(|out| { unsafe { neon_sys::object::get_own_property_names(out, self.to_raw()) } })
    }

    fn set<K: PropertyName, V: Any>(self, key: K, val: Handle<V>) -> Result<bool> {
        let mut result = false;
        if unsafe { key.set(&mut result, self.to_raw(), val.to_raw()) } {
            Ok(result)
        } else {
            Err(Throw)
        }
    }
}

impl Object for SomeObject { }

pub trait SomeObjectInternal {
    fn new_internal<'a>() -> Handle<'a, SomeObject>;
    fn build<'a, F: FnOnce(&mut raw::Local)>(init: F) -> Handle<'a, SomeObject>;
}

impl SomeObjectInternal for SomeObject {
    fn new_internal<'a>() -> Handle<'a, SomeObject> {
        SomeObject::build(|out| { unsafe { neon_sys::object::new(out) } })
    }

    fn build<'a, F: FnOnce(&mut raw::Local)>(init: F) -> Handle<'a, SomeObject> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            init(&mut local);
            Handle::new(SomeObject(local))
        }
    }
}


impl SomeObject {
    pub fn new<'a, T: Scope<'a>>(_: &mut T) -> Handle<'a, SomeObject> {
        SomeObject::new_internal()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Array(raw::Local);

impl Array {
    pub fn new<'a, T: Scope<'a>>(scope: &mut T, len: u32) -> Handle<'a, Array> {
        Array::new_internal(scope.isolate(), len)
    }
}

impl Any for Array { }

impl AnyInternal for Array {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { Array(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_array(other.to_raw()) }
    }
}

pub trait ArrayInternal {
    fn new_internal<'a>(isolate: *mut Isolate, len: u32) -> Handle<'a, Array>;
}

impl ArrayInternal for Array {
    fn new_internal<'a>(isolate: *mut Isolate, len: u32) -> Handle<'a, Array> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::array::new(&mut local, mem::transmute(isolate), len);
            Handle::new(Array(local))
        }
    }
}

impl Array {
    pub fn to_vec<'a, T: Scope<'a>>(self, scope: &mut T) -> Result<Vec<Handle<'a, Value>>> {
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

impl Object for Array { }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Function(raw::Local);

impl Function {
    pub fn new<'a, T: Scope<'a>, U: Any>(scope: &mut T, f: fn(Call) -> JS<U>) -> JS<'a, Function> {
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

extern "C" fn invoke_nanny_function<U: Any>(info: &CallbackInfo) {
    let mut scope = RootScope::new(unsafe { mem::transmute(neon_sys::call::get_isolate(mem::transmute(info))) });
    exec_function_body(info, &mut scope, |call| {
        let data = info.data();
        let kernel: fn(Call) -> JS<U> = unsafe { mem::transmute(neon_sys::fun::get_kernel(data.to_raw())) };
        if let Ok(value) = kernel(call) {
            info.set_return(value);
        }
    });
}

impl Any for Function { }

impl AnyInternal for Function {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { Function(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_function(other.to_raw()) }
    }
}
