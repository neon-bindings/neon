use std::mem;
use std::ffi::CStr;
use nanny_sys::raw;
use nanny_sys::{Nan_Export, Nan_NewObject, Nan_NewUndefined, Nan_NewNull, Nan_NewBoolean, Nan_NewInteger, Nan_NewNumber, Nan_NewArray, Nan_ArraySet};
use internal::local::{Local, LocalInternal};
use vm::{Call, Realm};

pub trait ValueInternal {
    fn to_raw_mut_ref(&mut self) -> &mut raw::Local;

    fn to_raw_ref(&self) -> &raw::Local;

    fn to_raw(&self) -> raw::Local {
        self.to_raw_ref().clone()
    }
}

pub trait Value: ValueInternal { }

#[repr(C)]
#[derive(Clone)]
pub struct Any(raw::Local);

impl Value for Any { }

impl ValueInternal for Any {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Any(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Any(ref mut local) = self;
        local
    }
}

pub trait AnyInternal {
    fn new<'a>(value: raw::Local) -> Local<'a, Any>;
}

impl AnyInternal for Any {
    fn new<'a>(value: raw::Local) -> Local<'a, Any> {
        Local::new(Any(value))
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Undefined(raw::Local);

impl Value for Undefined { }

impl ValueInternal for Undefined {
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
    fn new<'a>() -> Local<'a, Undefined>;
}

impl UndefinedInternal for Undefined {
    fn new<'a>() -> Local<'a, Undefined> {
        let mut result = Local::new(Undefined(unsafe { mem::zeroed() }));
        unsafe {
            Nan_NewUndefined(result.to_raw_mut_ref());
        }
        result
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Null(raw::Local);

impl Value for Null { }

impl ValueInternal for Null {
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
    fn new<'a>() -> Local<'a, Null>;
}

impl NullInternal for Null {
    fn new<'a>() -> Local<'a, Null> {
        let mut result = Local::new(Null(unsafe { mem::zeroed() }));
        unsafe {
            Nan_NewNull(result.to_raw_mut_ref());
        }
        result
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Boolean(raw::Local);

impl Value for Boolean { }

impl ValueInternal for Boolean {
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
    fn new<'a>(b: bool) -> Local<'a, Boolean>;
}

impl BooleanInternal for Boolean {
    fn new<'a>(b: bool) -> Local<'a, Boolean> {
        let mut result = Local::new(Boolean(unsafe { mem::zeroed() }));
        unsafe {
            Nan_NewBoolean(result.to_raw_mut_ref(), b);
        }
        result
    }
}

#[repr(C)]
pub struct String(raw::Local);

impl Value for String { }

impl ValueInternal for String {
    fn to_raw_ref(&self) -> &raw::Local {
        let &String(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut String(ref mut local) = self;
        local
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Integer(raw::Local);

impl Value for Integer { }

impl ValueInternal for Integer {
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
    fn new<'a, 'root>(realm: &'root Realm, i: i32) -> Local<'a, Integer>;
}

impl IntegerInternal for Integer {
    fn new<'a, 'root>(realm: &'root Realm, i: i32) -> Local<'a, Integer> {
        let mut result = Local::new(Integer(unsafe { mem::zeroed() }));
        unsafe {
            Nan_NewInteger(result.to_raw_mut_ref(), mem::transmute(realm), i);
        }
        result
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Number(raw::Local);

impl Value for Number { }

impl ValueInternal for Number {
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
    fn new<'a, 'root>(realm: &'root Realm, v: f64) -> Local<'a, Number>;
}

impl NumberInternal for Number {
    fn new<'a, 'root>(realm: &'root Realm, v: f64) -> Local<'a, Number> {
        let mut result = Local::new(Number(unsafe { mem::zeroed() }));
        unsafe {
            Nan_NewNumber(result.to_raw_mut_ref(), mem::transmute(realm), v);
        }
        result
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Object(raw::Local);

impl Value for Object { }

impl ValueInternal for Object {
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
    fn new<'a>() -> Local<'a, Object>;
}

impl ObjectInternal for Object {
    fn new<'a>() -> Local<'a, Object> {
        let mut result = Local::new(Object(unsafe { mem::zeroed() }));
        unsafe {
            Nan_NewObject(result.to_raw_mut_ref());
        }
        result
    }
}

impl Object {
    pub fn export(&mut self, name: &CStr, f: extern fn(&Call)) {
        let &mut Object(ref mut object) = self;
        unsafe {
            Nan_Export(object, mem::transmute(name.as_ptr()), mem::transmute(f));
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Array(raw::Local);

impl Value for Array { }

impl ValueInternal for Array {
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
    fn new<'a, 'root>(realm: &'root Realm, len: u32) -> Local<'a, Array>;
}

impl ArrayInternal for Array {
    fn new<'a, 'root>(realm: &'root Realm, len: u32) -> Local<'a, Array> {
        let mut result = Local::new(Array(unsafe { mem::zeroed() }));
        unsafe {
            Nan_NewArray(result.to_raw_mut_ref(), mem::transmute(realm), len);
        }
        result
    }
}

impl Array {
    pub fn set<'a, T: Clone + Value>(&mut self, index: u32, value: Local<'a, T>) -> bool {
        unsafe {
            Nan_ArraySet(self.to_raw_mut_ref(), index, value.to_raw())
        }
    }
}
