use std::mem;
use std::os::raw::c_void;
use std::ffi::{CString, CStr};
use nanny_sys::raw;
use nanny_sys::{Nan_NewObject, Nan_NewUndefined, Nan_NewNull, Nan_NewBoolean, Nan_NewInteger, Nan_NewString, Nan_NewNumber, Nan_NewArray, Node_ArrayLength, Nan_String_Utf8Length, Nan_Value_ToString, Nan_GetOwnPropertyNames, Nan_Get_Index, Nanny_Set_Index, Nan_Get, Nanny_Get_Bytes, Nanny_Set_Bytes, Nan_Set, Nanny_NewFunction, Nanny_FunctionKernel, Nan_FunctionCallbackInfo_GetIsolate, Nanny_IsUndefined, Nanny_IsNull, Nanny_IsInteger, Nanny_IsNumber, Nanny_IsString, Nanny_IsBoolean, Nanny_IsObject, Nanny_IsArray, Nanny_IsFunction, Nanny_TagOf, Tag};
use internal::mem::{Handle, HandleInternal};
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::vm::{Result, Throw, JS, Isolate, CallbackInfo, Call, exec_function_body};

pub trait AnyInternal: Copy {
    fn to_raw_mut_ref(&mut self) -> &mut raw::Local;

    fn to_raw_ref(&self) -> &raw::Local;

    fn to_raw(&self) -> raw::Local {
        self.to_raw_ref().clone()
    }

    fn is_typeof<Other: Any>(other: Other) -> bool;

    fn downcast<Other: Any>(other: Other) -> Option<Self> {
        if Self::is_typeof(other) {
            Some(Self::from_raw(other.to_raw()))
        } else {
            None
        }
    }

    fn cast<'a, 'b, T: Any, F: FnOnce(raw::Local) -> T>(&'a self, f: F) -> Handle<'b, T> {
        Handle::new(f(self.to_raw_ref().clone()))
    }

    fn from_raw(h: raw::Local) -> Self;
}

pub unsafe fn zeroed<'a, T: Any>() -> Handle<'a, T> {
    Handle::new(T::from_raw(mem::zeroed()))
}

pub fn build<'a, T: Any, F: FnOnce(&mut raw::Local) -> bool>(init: F) -> JS<'a, T> {
    unsafe {
        let mut result = zeroed::<T>();
        if init(result.to_raw_mut_ref()) {
            Ok(result)
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
    fn to_string<'a, T: Scope<'a>>(&mut self, _: &mut T) -> JS<'a, String> {
        build(|out| { unsafe { Nan_Value_ToString(out, self.to_raw_mut_ref()) } })
    }

    fn value<'a, T: Scope<'a>>(&self, _: &mut T) -> Handle<'a, Value> {
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
    fn to_raw_ref(&self) -> &raw::Local {
        let &Value(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Value(ref mut local) = self;
        local
    }

    fn from_raw(h: raw::Local) -> Self { Value(h) }

    fn is_typeof<Other: Any>(_: Other) -> bool {
        true
    }
}

impl<'a> Handle<'a, Value> {
    pub fn variant(&self) -> Variant<'a> {
        match unsafe { Nanny_TagOf(self.to_raw_ref()) } {
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
    fn to_raw_ref(&self) -> &raw::Local {
        let &Undefined(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Undefined(ref mut local) = self;
        local
    }

    fn from_raw(h: raw::Local) -> Self { Undefined(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { Nanny_IsUndefined(other.to_raw_ref()) }
    }
}

pub trait UndefinedInternal {
    fn new_internal<'a>() -> Handle<'a, Undefined>;
}

impl UndefinedInternal for Undefined {
    fn new_internal<'a>() -> Handle<'a, Undefined> {
        let mut result = unsafe { zeroed::<Undefined>() };
        unsafe {
            Nan_NewUndefined(result.to_raw_mut_ref());
        }
        result
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
    fn to_raw_ref(&self) -> &raw::Local {
        let &Null(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Null(ref mut local) = self;
        local
    }

    fn from_raw(h: raw::Local) -> Self { Null(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { Nanny_IsNull(other.to_raw_ref()) }
    }
}

pub trait NullInternal {
    fn new_internal<'a>() -> Handle<'a, Null>;
}

impl NullInternal for Null {
    fn new_internal<'a>() -> Handle<'a, Null> {
        let mut result = unsafe { zeroed::<Null>() };
        unsafe {
            Nan_NewNull(result.to_raw_mut_ref());
        }
        result
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
    fn to_raw_ref(&self) -> &raw::Local {
        let &Boolean(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Boolean(ref mut local) = self;
        local
    }

    fn from_raw(h: raw::Local) -> Self { Boolean(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { Nanny_IsBoolean(other.to_raw_ref()) }
    }
}

pub trait BooleanInternal {
    fn new_internal<'a>(b: bool) -> Handle<'a, Boolean>;
}

impl BooleanInternal for Boolean {
    fn new_internal<'a>(b: bool) -> Handle<'a, Boolean> {
        let mut result = unsafe { zeroed::<Boolean>() };
        unsafe {
            Nan_NewBoolean(result.to_raw_mut_ref(), b);
        }
        result
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct String(raw::Local);

impl Any for String { }

impl AnyInternal for String {
    fn to_raw_ref(&self) -> &raw::Local {
        let &String(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut String(ref mut local) = self;
        local
    }

    fn from_raw(h: raw::Local) -> Self { String(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { Nanny_IsString(other.to_raw_ref()) }
    }
}

impl String {
    pub fn size(&self) -> isize {
        unsafe {
            Nan_String_Utf8Length(self.to_raw_ref())
        }
    }

    pub fn new<'a, T: Scope<'a>>(scope: &mut T, val: &str) -> Option<Handle<'a, String>> {
        CString::new(val).ok().and_then(|str| String::new_internal(scope.isolate(), &str))
    }
}

pub trait StringInternal {
    fn new_internal<'a>(isolate: *mut Isolate, val: &CStr) -> Option<Handle<'a, String>>;
}

// Lower a &CStr to the types expected by Node: a const *uint8_t buffer and an int32_t length.
fn lower_cstr(cs: &CStr) -> Option<(*const u8, i32)> {
    // V8 currently refuses to allocate strings longer than `(1 << 20) - 16` bytes,
    // but in case this changes over time, just ensure the buffer isn't longer than
    // the largest positive signed integer, and delegate the tighter bounds checks
    // to V8.
    let len = cs.to_bytes().len();
    if len > (::std::i32::MAX as usize) {
        return None;
    }
    Some((unsafe { mem::transmute(cs.as_ptr()) }, len as i32))
}

fn lower_cstr_unwrap(cs: &CStr) -> (*const u8, i32) {
    lower_cstr(cs).unwrap_or_else(|| {
        panic!("{} < i32::MAX", cs.to_bytes().len())
    })
}

fn lower_str_unwrap(s: &str) -> (*const u8, i32) {
    lower_cstr_unwrap(&CString::new(s).ok().unwrap())
}

impl StringInternal for String {
    fn new_internal<'a>(isolate: *mut Isolate, val: &CStr) -> Option<Handle<'a, String>> {
        let (ptr, len) = match lower_cstr(val) {
            Some(pair) => pair,
            None => { return None; }
        };
        unsafe {
            let mut result = zeroed::<String>();
            // FIXME: this is currently traversing the string twice (see the note in the CStr::as_ptr docs)
            if Nan_NewString(result.to_raw_mut_ref(), mem::transmute(isolate), ptr, len) {
                Some(result)
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
    fn to_raw_ref(&self) -> &raw::Local {
        let &Integer(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Integer(ref mut local) = self;
        local
    }

    fn from_raw(h: raw::Local) -> Self { Integer(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { Nanny_IsInteger(other.to_raw_ref()) }
    }
}

pub trait IntegerInternal {
    fn new_internal<'a>(isolate: *mut Isolate, i: i32) -> Handle<'a, Integer>;
}

impl IntegerInternal for Integer {
    fn new_internal<'a>(isolate: *mut Isolate, i: i32) -> Handle<'a, Integer> {
        let mut result = unsafe { zeroed::<Integer>() };
        unsafe {
            Nan_NewInteger(result.to_raw_mut_ref(), mem::transmute(isolate), i);
        }
        result
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
    fn to_raw_ref(&self) -> &raw::Local {
        let &Number(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Number(ref mut local) = self;
        local
    }

    fn from_raw(h: raw::Local) -> Self { Number(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { Nanny_IsNumber(other.to_raw_ref()) }
    }
}

pub trait NumberInternal {
    fn new_internal<'a>(isolate: *mut Isolate, v: f64) -> Handle<'a, Number>;
}

impl NumberInternal for Number {
    fn new_internal<'a>(isolate: *mut Isolate, v: f64) -> Handle<'a, Number> {
        let mut result = unsafe { zeroed::<Number>() };
        unsafe {
            Nan_NewNumber(result.to_raw_mut_ref(), mem::transmute(isolate), v);
        }
        result
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SomeObject(raw::Local);

impl Any for SomeObject { }

impl AnyInternal for SomeObject {
    fn to_raw_ref(&self) -> &raw::Local {
        let &SomeObject(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut SomeObject(ref mut local) = self;
        local
    }

    fn from_raw(h: raw::Local) -> Self { SomeObject(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { Nanny_IsObject(other.to_raw_ref()) }
    }
}

trait PropertyName {
    unsafe fn get(self, out: &mut raw::Local, obj: &mut raw::Local) -> bool;
    unsafe fn set(self, out: &mut bool, key: &mut raw::Local, val: &mut raw::Local) -> bool;
}

impl PropertyName for u32 {
    unsafe fn get(self, out: &mut raw::Local, obj: &mut raw::Local) -> bool {
        Nan_Get_Index(out, obj, self)
    }

    unsafe fn set(self, out: &mut bool, obj: &mut raw::Local, val: &mut raw::Local) -> bool {
        Nanny_Set_Index(out, obj, self, val)
    }
}

impl<'a, K: Any> PropertyName for Handle<'a, K> {
    unsafe fn get(mut self, out: &mut raw::Local, obj: &mut raw::Local) -> bool {
        Nan_Get(out, obj, self.to_raw_mut_ref())
    }

    unsafe fn set(mut self, out: &mut bool, obj: &mut raw::Local, val: &mut raw::Local) -> bool {
        Nan_Set(out, obj, self.to_raw_mut_ref(), val)
    }
}

impl<'a> PropertyName for &'a str {
    unsafe fn get(self, out: &mut raw::Local, obj: &mut raw::Local) -> bool {
        let (ptr, len) = lower_str_unwrap(self);
        Nanny_Get_Bytes(out, obj, ptr, len)
    }

    unsafe fn set(self, out: &mut bool, obj: &mut raw::Local, val: &mut raw::Local) -> bool {
        let (ptr, len) = lower_str_unwrap(self);
        Nanny_Set_Bytes(out, obj, ptr, len, val)
    }
}

pub trait Object: Any {
    fn get<'a, T: Scope<'a>, K: PropertyName>(&mut self, _: &mut T, key: K) -> Result<Handle<'a, Value>> {
        build(|out| { unsafe { key.get(out, self.to_raw_mut_ref()) } })
    }

    fn get_own_property_names<'a, T: Scope<'a>>(&self, _: &mut T) -> JS<'a, Array> {
        build(|out| { unsafe { Nan_GetOwnPropertyNames(out, self.to_raw_ref()) } })
    }

    fn set<K: PropertyName, V: Any>(&mut self, key: K, mut val: Handle<V>) -> Result<bool> {
        let mut result = false;
        if unsafe { key.set(&mut result, self.to_raw_mut_ref(), val.to_raw_mut_ref()) } {
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
        SomeObject::build(|out| { unsafe { Nan_NewObject(out) } })
    }

    fn build<'a, F: FnOnce(&mut raw::Local)>(init: F) -> Handle<'a, SomeObject> {
        unsafe {
            let mut result = zeroed::<SomeObject>();
            init(result.to_raw_mut_ref());
            result
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
    fn to_raw_ref(&self) -> &raw::Local {
        let &Array(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Array(ref mut local) = self;
        local
    }

    fn from_raw(h: raw::Local) -> Self { Array(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { Nanny_IsArray(other.to_raw_ref()) }
    }
}

pub trait ArrayInternal {
    fn new_internal<'a>(isolate: *mut Isolate, len: u32) -> Handle<'a, Array>;
}

impl ArrayInternal for Array {
    fn new_internal<'a>(isolate: *mut Isolate, len: u32) -> Handle<'a, Array> {
        let mut result = unsafe { zeroed::<Array>() };
        unsafe {
            Nan_NewArray(result.to_raw_mut_ref(), mem::transmute(isolate), len);
        }
        result
    }
}

impl Array {
    pub fn to_vec<'a, T: Scope<'a>>(&mut self, scope: &mut T) -> Result<Vec<Handle<'a, Value>>> {
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

    pub fn len(&self) -> u32 {
        unsafe {
            Node_ArrayLength(self.to_raw_ref())
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
                Nanny_NewFunction(out, isolate, callback, kernel)
            }
        })
    }
}

extern "C" fn invoke_nanny_function<U: Any>(info: &CallbackInfo) {
    let mut scope = RootScope::new(unsafe { mem::transmute(Nan_FunctionCallbackInfo_GetIsolate(mem::transmute(info))) });
    exec_function_body(info, &mut scope, |call| {
        let data = info.data();
        let kernel: fn(Call) -> JS<U> = unsafe { mem::transmute(Nanny_FunctionKernel(data.to_raw_ref())) };
        if let Ok(value) = kernel(call) {
            info.set_return(value);
        }
    });
}

impl Any for Function { }

impl AnyInternal for Function {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Function(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Function(ref mut local) = self;
        local
    }

    fn from_raw(h: raw::Local) -> Self { Function(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { Nanny_IsFunction(other.to_raw_ref()) }
    }
}
