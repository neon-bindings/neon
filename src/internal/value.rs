use std::mem;
use std::os::raw::c_void;
use std::ffi::{CString, CStr};
use nanny_sys::raw;
use nanny_sys::{Nan_NewObject, Nan_NewUndefined, Nan_NewNull, Nan_NewBoolean, Nan_NewInteger, Nan_NewString, Nan_NewNumber, Nan_NewArray, Node_ArraySet, Node_ArrayLength, Nan_String_Utf8Length, Nan_Value_ToString, Nan_Value_ToObject, Nan_GetOwnPropertyNames, Nan_Get_Index, Nan_Get, Nan_Set, Nanny_NewFunction, Nanny_FunctionKernel, Nan_FunctionCallbackInfo_GetIsolate};
use internal::mem::{Handle, HandleInternal};
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::vm::{Result, Throw, JS, Isolate, CallbackInfo, Call, exec_function_body};

pub trait TaggedInternal {
    fn to_raw_mut_ref(&mut self) -> &mut raw::Local;

    fn to_raw_ref(&self) -> &raw::Local;

    fn to_raw(&self) -> raw::Local {
        self.to_raw_ref().clone()
    }

    fn cast<'a, 'b, T: Copy + Tagged, F: FnOnce(raw::Local) -> T>(&'a self, f: F) -> Handle<'b, T> {
        Handle::new(f(self.to_raw_ref().clone()))
    }
}

pub trait Tagged: TaggedInternal {
    fn to_string<'fun, 'block, 'scope, T: Scope<'fun, 'block>>(&mut self, _: &'scope mut T) -> JS<'block, String> {
        // FIXME: String could use a build_opt abstraction too
        unsafe {
            let mut result = Handle::new(String(mem::zeroed()));
            if Nan_Value_ToString(result.to_raw_mut_ref(), self.to_raw_mut_ref()) {
                Ok(result)
            } else {
                Err(Throw)
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Value(raw::Local);

impl Tagged for Value { }

impl TaggedInternal for Value {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Value(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Value(ref mut local) = self;
        local
    }
}

impl Value {
    pub fn as_object<'fun, 'block, 'scope, T: Scope<'fun, 'block>>(&self, _: &'scope mut T) -> Option<Handle<'block, Object>> {
        Object::build_opt(|out| { unsafe { Nan_Value_ToObject(out, self.to_raw_ref()) } })
    }

    pub fn check_object<'fun, 'block, 'scope, T: Scope<'fun, 'block>>(&self, _: &'scope mut T) -> JS<'block, Object> {
        Object::build_opt(|out| { unsafe { Nan_Value_ToObject(out, self.to_raw_ref()) } })
            .ok_or_else(|| {
                // FIXME: throw a type error
                Throw
            })
    }
}

pub trait ValueInternal {
    fn new_internal<'a>(value: raw::Local) -> Handle<'a, Value>;
    unsafe fn zero_internal<'a>() -> Handle<'a, Value>;
}

impl ValueInternal for Value {
    fn new_internal<'a>(value: raw::Local) -> Handle<'a, Value> {
        Handle::new(Value(value))
    }

    unsafe fn zero_internal<'a>() -> Handle<'a, Value> {
        Handle::new(Value(mem::zeroed()))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Undefined(raw::Local);

impl Undefined {
    pub fn new<'fun, 'block, T: Scope<'fun, 'block>>(_: &mut T) -> Handle<'block, Undefined> {
        Undefined::new_internal()
    }
}

impl Tagged for Undefined { }

impl TaggedInternal for Undefined {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Undefined(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Undefined(ref mut local) = self;
        local
    }
}

pub trait UndefinedInternal {
    fn new_internal<'a>() -> Handle<'a, Undefined>;
}

impl UndefinedInternal for Undefined {
    fn new_internal<'a>() -> Handle<'a, Undefined> {
        let mut result = Handle::new(Undefined(unsafe { mem::zeroed() }));
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
    pub fn new<'fun, 'block, T: Scope<'fun, 'block>>(_: &mut T) -> Handle<'block, Null> {
        Null::new_internal()
    }
}

impl Tagged for Null { }

impl TaggedInternal for Null {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Null(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Null(ref mut local) = self;
        local
    }
}

pub trait NullInternal {
    fn new_internal<'a>() -> Handle<'a, Null>;
}

impl NullInternal for Null {
    fn new_internal<'a>() -> Handle<'a, Null> {
        let mut result = Handle::new(Null(unsafe { mem::zeroed() }));
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
    pub fn new<'fun, 'block, T: Scope<'fun, 'block>>(_: &mut T, b: bool) -> Handle<'block, Boolean> {
        Boolean::new_internal(b)
    }
}

impl Tagged for Boolean { }

impl TaggedInternal for Boolean {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Boolean(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Boolean(ref mut local) = self;
        local
    }
}

pub trait BooleanInternal {
    fn new_internal<'a>(b: bool) -> Handle<'a, Boolean>;
}

impl BooleanInternal for Boolean {
    fn new_internal<'a>(b: bool) -> Handle<'a, Boolean> {
        let mut result = Handle::new(Boolean(unsafe { mem::zeroed() }));
        unsafe {
            Nan_NewBoolean(result.to_raw_mut_ref(), b);
        }
        result
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct String(raw::Local);

impl Tagged for String { }

impl TaggedInternal for String {
    fn to_raw_ref(&self) -> &raw::Local {
        let &String(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut String(ref mut local) = self;
        local
    }
}

impl String {
    pub fn size(&self) -> isize {
        unsafe {
            Nan_String_Utf8Length(self.to_raw_ref())
        }
    }

    pub fn new<'fun, 'block, T: Scope<'fun, 'block>>(scope: &mut T, val: &str) -> Option<Handle<'block, String>> {
        CString::new(val).ok().and_then(|str| String::new_internal(scope.isolate(), &str))
    }
}

pub trait StringInternal {
    fn new_internal<'a, 'fun>(isolate: &'fun Isolate, val: &CStr) -> Option<Handle<'a, String>>;
}

impl StringInternal for String {
    fn new_internal<'a, 'fun>(isolate: &'fun Isolate, val: &CStr) -> Option<Handle<'a, String>> {
        unsafe {
            let mut result = Handle::new(String(mem::zeroed()));
            // FIXME: this is currently traversing the string twice (see the note in the CStr::as_ptr docs)
            // FIXME: range check on length?
            if Nan_NewString(result.to_raw_mut_ref(), mem::transmute(isolate), mem::transmute(val.as_ptr()), val.to_bytes().len() as i32) {
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
    pub fn new<'fun, 'block, T: Scope<'fun, 'block>>(scope: &mut T, i: i32) -> Handle<'block, Integer> {
        Integer::new_internal(scope.isolate(), i)
    }
}

impl Tagged for Integer { }

impl TaggedInternal for Integer {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Integer(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Integer(ref mut local) = self;
        local
    }
}

pub trait IntegerInternal {
    fn new_internal<'a, 'fun>(isolate: &'fun Isolate, i: i32) -> Handle<'a, Integer>;
}

impl IntegerInternal for Integer {
    fn new_internal<'a, 'fun>(isolate: &'fun Isolate, i: i32) -> Handle<'a, Integer> {
        let mut result = Handle::new(Integer(unsafe { mem::zeroed() }));
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
    pub fn new<'fun, 'block, T: Scope<'fun, 'block>>(scope: &mut T, v: f64) -> Handle<'block, Number> {
        Number::new_internal(scope.isolate(), v)
    }
}

impl Tagged for Number { }

impl TaggedInternal for Number {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Number(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Number(ref mut local) = self;
        local
    }
}

pub trait NumberInternal {
    fn new_internal<'a, 'fun>(isolate: &'fun Isolate, v: f64) -> Handle<'a, Number>;
}

impl NumberInternal for Number {
    fn new_internal<'a, 'fun>(isolate: &'fun Isolate, v: f64) -> Handle<'a, Number> {
        let mut result = Handle::new(Number(unsafe { mem::zeroed() }));
        unsafe {
            Nan_NewNumber(result.to_raw_mut_ref(), mem::transmute(isolate), v);
        }
        result
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Object(raw::Local);

impl Object {
    // FIXME: shouldn't this be fallible?
    pub fn new<'fun, 'block, T: Scope<'fun, 'block>>(_: &mut T) -> Handle<'block, Object> {
        Object::new_internal()
    }

    pub fn get_own_property_names<'fun, 'block, T: Scope<'fun, 'block>>(&self, _: &mut T) -> JS<'block, Array> {
        // FIXME: Array could use a build_opt abstraction too
        unsafe {
            let mut result = Handle::new(Array(mem::zeroed()));
            if Nan_GetOwnPropertyNames(result.to_raw_mut_ref(), self.to_raw_ref()) {
                Ok(result)
            } else {
                Err(Throw)
            }
        }
    }
}

impl Tagged for Object { }

impl TaggedInternal for Object {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Object(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Object(ref mut local) = self;
        local
    }
}

pub trait ObjectInternal {
    fn new_internal<'a>() -> Handle<'a, Object>;
    unsafe fn zero_internal<'a>() -> Handle<'a, Object>;
    fn build<'a, F: FnOnce(&mut raw::Local)>(init: F) -> Handle<'a, Object>;
    fn build_opt<'a, F: FnOnce(&mut raw::Local) -> bool>(init: F) -> Option<Handle<'a, Object>>;
}

impl ObjectInternal for Object {
    unsafe fn zero_internal<'a>() -> Handle<'a, Object> {
        Handle::new(Object(mem::zeroed()))
    }

    fn new_internal<'a>() -> Handle<'a, Object> {
        Object::build(|out| { unsafe { Nan_NewObject(out) } })
    }

    fn build<'a, F: FnOnce(&mut raw::Local)>(init: F) -> Handle<'a, Object> {
        unsafe {
            let mut result = Object::zero_internal();
            init(result.to_raw_mut_ref());
            result
        }
    }

    fn build_opt<'a, F: FnOnce(&mut raw::Local) -> bool>(init: F) -> Option<Handle<'a, Object>> {
        unsafe {
            let mut result = Object::zero_internal();
            if init(result.to_raw_mut_ref()) {
                Some(result)
            } else {
                None
            }
        }
    }
}


impl Object {
    // FIXME: make get/set overloadable with a `PropertyName` trait that has private unsafe get/set methods
    // FIXME: make it generic instead of Value
    pub fn get<'fun, 'block, T: Scope<'fun, 'block>>(&mut self, _: &mut T, mut key: Handle<Value>) -> JS<'block, Value> {
        unsafe {
            // FIXME: could use a Value build_opt
            let mut result = Value::zero_internal();
            if Nan_Get(result.to_raw_mut_ref(), self.to_raw_mut_ref(), key.to_raw_mut_ref()) {
                Ok(result)
            } else {
                Err(Throw)
            }
        }
    }

    // FIXME: overloadable with a `PropertyName` trait
    // FIXME: make it generic instead of Value
    pub fn set<'fun, 'block, T: Scope<'fun, 'block>>(&mut self, scope: &mut T, key: &str, val: Handle<Value>) -> Result<bool> {
        let mut key = try!(String::new(scope, key).ok_or(Throw));
        let mut result = false;
        if unsafe { Nan_Set(&mut result, self.to_raw_mut_ref(), key.to_raw_mut_ref(), val.to_raw_ref()) } {
            Ok(result)
        } else {
            Err(Throw)
        }
    }
}

// FIXME: replace `upcast` with infallible to_object, to_value methods

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Array(raw::Local);

impl Array {
    pub fn new<'fun, 'block, T: Scope<'fun, 'block>>(scope: &mut T, len: u32) -> Handle<'block, Array> {
        Array::new_internal(scope.isolate(), len)
    }
}

impl Tagged for Array { }

impl TaggedInternal for Array {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Array(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Array(ref mut local) = self;
        local
    }
}

pub trait ArrayInternal {
    fn new_internal<'a, 'fun>(isolate: &'fun Isolate, len: u32) -> Handle<'a, Array>;
}

impl ArrayInternal for Array {
    fn new_internal<'a, 'fun>(isolate: &'fun Isolate, len: u32) -> Handle<'a, Array> {
        let mut result = Handle::new(Array(unsafe { mem::zeroed() }));
        unsafe {
            Nan_NewArray(result.to_raw_mut_ref(), mem::transmute(isolate), len);
        }
        result
    }
}

impl Array {
    pub fn set<'a, T: Copy + Tagged>(&mut self, index: u32, value: Handle<'a, T>) -> bool {
        unsafe {
            Node_ArraySet(self.to_raw_mut_ref(), index, value.to_raw())
        }
    }

    pub fn get_index<'fun, 'block, T: Scope<'fun, 'block>>(&mut self, _: &mut T, index: u32) -> Option<Handle<'block, Value>> {
        unsafe {
            // FIXME: could use a Value build_opt
            let mut result = Value::zero_internal();
            if Nan_Get_Index(result.to_raw_mut_ref(), self.to_raw_mut_ref(), index) {
                Some(result)
            } else {
                None
            }
        }
    }

    pub fn to_vec<'fun, 'block, T: Scope<'fun, 'block>>(&mut self, scope: &mut T) -> Result<Vec<Handle<'block, Value>>> {
        let mut result = Vec::with_capacity(self.len() as usize);
        let mut i = 0;
        loop {
            // Since getting a property can trigger arbitrary code,
            // we have to re-check the length on every iteration.
            if i >= self.len() {
                return Ok(result);
            }
            match self.get_index(scope, i) {
                Some(val) => { result.push(val); }
                None => { return Err(Throw); }
            }
            i += 1;
        }
    }

    pub fn len(&self) -> u32 {
        unsafe {
            Node_ArrayLength(self.to_raw_ref())
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Function(raw::Local);

impl Function {
    pub fn new<'fun, 'block, T: Scope<'fun, 'block>, U: Copy + Tagged>(scope: &mut T, f: fn(Call) -> JS<U>) -> Option<Handle<'block, Function>> {
        unsafe {
            let mut result = Function::zero_internal();
            let isolate: *mut c_void = mem::transmute(scope.isolate());
            let callback: extern "C" fn(&CallbackInfo) = invoke_nanny_function::<U>;
            let callback: *mut c_void = mem::transmute(callback);
            let kernel: *mut c_void = mem::transmute(f);
            if Nanny_NewFunction(result.to_raw_mut_ref(), isolate, callback, kernel) {
                Some(result)
            } else {
                None
            }
        }
    }
}

pub trait FunctionInternal {
    unsafe fn zero_internal<'a>() -> Handle<'a, Function>;
}

impl FunctionInternal for Function {
    unsafe fn zero_internal<'a>() -> Handle<'a, Function> {
        Handle::new(Function(mem::zeroed()))
    }
}

extern "C" fn invoke_nanny_function<U: Copy + Tagged>(info: &CallbackInfo) {
    let mut scope = RootScope::new(unsafe { mem::transmute(Nan_FunctionCallbackInfo_GetIsolate(mem::transmute(info))) });
    exec_function_body(info, &mut scope, |call| {
        let data = info.data();
        let kernel: fn(Call) -> JS<U> = unsafe { mem::transmute(Nanny_FunctionKernel(data.to_raw_ref())) };
        if let Ok(value) = kernel(call) {
            info.set_return(value);
        }
    });
}

impl Tagged for Function { }

impl TaggedInternal for Function {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Function(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Function(ref mut local) = self;
        local
    }
}
