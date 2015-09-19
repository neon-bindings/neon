extern crate nanny_sys;

use std::fmt::Debug;
use std::mem;
use std::os::raw::c_void;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ffi::CStr;
use nanny_sys::raw;
use nanny_sys::{Nan_FunctionCallbackInfo_SetReturnValue, Nan_Export, Nan_NewObject, /*Nan_MaybeLocalString_ToOption, Nan_MaybeLocalString_IsEmpty,*/ Nan_Scoped, Nan_EscapeScoped, Nan_NewInteger, Nan_NewNumber, Nan_NewArray, Nan_ArraySet};

#[repr(C)]
pub struct FunctionCallbackInfo(raw::FunctionCallbackInfo);

impl FunctionCallbackInfo {
    // GC: Storing a Local in a ReturnValue keeps it alive independent of any HandleScope.
    pub fn set_return<'a, 'b, T: Clone + Value>(&'a mut self, value: Local<'b, T>) {
        let &mut FunctionCallbackInfo(ref mut info) = self;
        unsafe {
            Nan_FunctionCallbackInfo_SetReturnValue(info, value.to_raw());
        }
    }
}

pub trait Value {
    fn to_raw(&self) -> raw::Local;
}

pub trait Handler {
    fn integer(&self, i: i32) -> Local<Integer> {
        Integer::new(i)
    }

    fn number(&self, v: f64) -> Local<Number> {
        Number::new(v)
    }

    fn array(&self, len: u32) -> Local<Array> {
        Array::new(len)
    }

    fn object(&self) -> Local<Object> {
        Object::new()
    }
}

#[repr(C)]
pub struct Local<'a, T: Clone + Value + 'a> {
    value: T,
    phantom: PhantomData<&'a T>
}

impl<'a, T: Clone + Value> Deref for Local<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        &self.value
    }
}

impl<'a, T: Clone + Value> DerefMut for Local<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut self.value
    }
}

#[repr(C)]
pub struct String(raw::Local);

impl Value for String {
    fn to_raw(&self) -> raw::Local {
        let &String(ref local) = self;
        local.clone()
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Integer(raw::Local);

impl Value for Integer {
    fn to_raw(&self) -> raw::Local {
        let &Integer(ref local) = self;
        local.clone()
    }
}

impl Integer {
    fn new<'a>(i: i32) -> Local<'a, Integer> {
        let mut result = Local {
            value: Integer(unsafe { mem::uninitialized() }),
            phantom: PhantomData
        };
        match &mut result {
            &mut Local { value: Integer(ref mut integer), .. } => {
                unsafe {
                    Nan_NewInteger(integer, i);
                }
            }
        }
        result
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Number(raw::Local);

impl Value for Number {
    fn to_raw(&self) -> raw::Local {
        let &Number(ref local) = self;
        local.clone()
    }
}

impl Number {
    fn new<'a>(v: f64) -> Local<'a, Number> {
        let mut result = Local {
            value: Number(unsafe { mem::uninitialized() }),
            phantom: PhantomData
        };
        match &mut result {
            &mut Local { value: Number(ref mut number), .. } => {
                unsafe {
                    Nan_NewNumber(number, v);
                }
            }
        }
        result
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Object(raw::Local);

impl Value for Object {
    fn to_raw(&self) -> raw::Local {
        let &Object(ref local) = self;
        local.clone()
    }
}

impl Object {
    fn new<'a>() -> Local<'a, Object> {
        let mut result = Local {
            value: Object(unsafe { mem::uninitialized() }),
            phantom: PhantomData
        };
        match &mut result {
            &mut Local { value: Object(ref mut object), .. } => {
                unsafe {
                    Nan_NewObject(object);
                }
            }
        }
        result
    }

    pub fn export(&mut self, name: &CStr, f: extern fn(&mut FunctionCallbackInfo)) {
        let &mut Object(ref mut object) = self;
        unsafe {
            Nan_Export(object, mem::transmute(name.as_ptr()), mem::transmute(f));
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Array(raw::Local);

impl Value for Array {
    fn to_raw(&self) -> raw::Local {
        let &Array(ref local) = self;
        local.clone()
    }
}

impl Array {
    fn new<'a>(len: u32) -> Local<'a, Array> {
        let mut result = Local {
            value: Array(unsafe { mem::uninitialized() }),
            phantom: PhantomData
        };
        match &mut result {
            &mut Local { value: Array(ref mut array), .. } => {
                unsafe {
                    Nan_NewArray(array, len);
                }
            }
        }
        result
    }

    pub fn set<'a, T: Clone + Value>(&mut self, index: u32, value: Local<'a, T>) -> bool {
        match self {
            &mut Array(ref mut array) => {
                unsafe {
                    Nan_ArraySet(array, index, value.to_raw())
                }
            }
        }
    }
}

pub struct EscapeScope;

impl Handler for EscapeScope { }

impl EscapeScope {
    pub fn run<T: Debug, F: FnOnce(&EscapeScope) -> T>(f: F) -> T {
        // FIXME: too much copy-paste with Scope -- how much can be abstracted?
        unsafe {
            let closure: Box<F> = Box::new(f);
            let callback: extern "C" fn(&mut Box<Option<T>>, &EscapeScope, Box<F>) = escape_scope_run_helper::<T, F>;
            let closure: *mut c_void = mem::transmute(closure);
            let callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
            // I tried avoiding the option-wrapping with mem::uninitialized, but Box::new(mem::uninitialized)
            // seems to cause some sort of crash, presumably because it has to copy uninitialized memory.
            let mut result: Box<Option<T>> = Box::new(None);
            {
                let out: &mut Box<Option<T>> = &mut result;
                let out: *mut c_void = mem::transmute(out);
                Nan_EscapeScoped(out, closure, callback);
            }
            result.unwrap()
        }
    }

    pub fn escape<'a, 'b, T: Clone + Value>(&'a mut self, local: Local<'a, T>) -> Local<'b, T> {
        Local {
            value: local.value.clone(),
            phantom: PhantomData
        }
    }
}

pub struct Scope;

impl Handler for Scope { }

impl Scope {
    pub fn run<T: Debug, F: FnOnce(&Scope) -> T>(f: F) -> T {
        unsafe {
            let closure: Box<F> = Box::new(f);
            let callback: extern "C" fn(&mut Box<Option<T>>, &Scope, Box<F>) = scope_run_helper::<T, F>;
            let closure: *mut c_void = mem::transmute(closure);
            let callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
            // I tried avoiding the option-wrapping with mem::uninitialized, but Box::new(mem::uninitialized)
            // seems to cause some sort of crash, presumably because it has to copy uninitialized memory.
            let mut result: Box<Option<T>> = Box::new(None);
            {
                let out: &mut Box<Option<T>> = &mut result;
                let out: *mut c_void = mem::transmute(out);
                Nan_Scoped(out, closure, callback);
            }
            result.unwrap()
        }
    }
}

extern "C" fn scope_run_helper<T: Debug, F: FnOnce(&Scope) -> T>(out: &mut Box<Option<T>>, scope: &Scope, f: Box<F>) {
    let result = f(scope);
    **out = Some(result);
}

extern "C" fn escape_scope_run_helper<T: Debug, F: FnOnce(&EscapeScope) -> T>(out: &mut Box<Option<T>>, scope: &EscapeScope, f: Box<F>) {
    let result = f(scope);
    **out = Some(result);
}

/*
#[repr(C)]
pub struct LocalObject(raw::LocalObject);

impl LocalObject {
    pub fn new() -> LocalObject {
        unsafe {
            LocalObject(Nan_NewObject())
        }
    }

    pub fn export(&mut self, name: &CStr, f: extern fn(&mut FunctionCallbackInfo)) {
        let &mut LocalObject(ref mut object) = self;
        unsafe {
            Nan_Export(object, mem::transmute(name.as_ptr()), mem::transmute(f));
        }
    }
}
 */

/*
#[repr(C)]
pub struct LocalString(raw::LocalString);

trait RawMaybeLocalStringExt {
    fn is_empty(&self) -> bool;
    fn to_option(&self) -> Option<LocalString>;
}

impl RawMaybeLocalStringExt for raw::MaybeLocalString {
    fn is_empty(&self) -> bool {
        unsafe {
            Nan_MaybeLocalString_IsEmpty(self)
        }
    }

    fn to_option(&self) -> Option<LocalString> {
        unsafe {
            let mut tmp: raw::LocalString = mem::uninitialized();
            if Nan_MaybeLocalString_ToOption(self, &mut tmp) {
                Some(LocalString(tmp.clone()))
            } else {
                None
            }
        }
    }
}
 */
