extern crate nanny_sys;

use std::fmt::Debug;
use std::mem;
use std::os::raw::c_void;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ffi::CStr;
use std::cell::{RefCell, UnsafeCell};
use nanny_sys::raw;
use nanny_sys::{Nan_FunctionCallbackInfo_SetReturnValue, Nan_FunctionCallbackInfo_GetIsolate, Nan_Export, Nan_NewObject, /*Nan_MaybeLocalString_ToOption, Nan_MaybeLocalString_IsEmpty,*/ Nan_Root, Nan_Nested, Nan_Chained, Nan_NewUndefined, Nan_NewNull, Nan_NewInteger, Nan_NewNumber, Nan_NewArray, Nan_ArraySet, Nan_EscapableHandleScope_Escape};

// FIXME: split this up into a number of separate modules (values, locals, vm, scopes)

#[repr(C)]
pub struct Call {
    activation: UnsafeCell<Activation>
}

#[repr(C)]
pub struct Activation {
    info: raw::FunctionCallbackInfo
}

impl Call {
    pub fn activation(&self) -> &mut Activation {
        unsafe {
            mem::transmute(self.activation.get())
        }
    }

    pub fn realm(&self) -> &Realm {
        unsafe {
            mem::transmute(Nan_FunctionCallbackInfo_GetIsolate(&(*(self.activation.get())).info))
        }
    }
}

impl Activation {
    // GC: Storing a Local in a ReturnValue keeps it alive independent of any HandleScope.
    pub fn set_return<'a, 'b, T: Clone + Value>(&'a mut self, value: Local<'b, T>) {
         unsafe {
            Nan_FunctionCallbackInfo_SetReturnValue(&mut self.info, value.to_raw());
        }
    }
}

pub trait Value {
    // FIXME: put this in a private subtrait?
    fn to_raw_mut_ref(&mut self) -> &mut raw::Local;
    fn to_raw_ref(&self) -> &raw::Local;

    fn to_raw(&self) -> raw::Local {
        self.to_raw_ref().clone()
    }
}

#[repr(C)]
pub struct Local<'a, T: Clone + Value + 'a> {
    value: T,
    phantom: PhantomData<&'a T>
}

impl<'a, T: Clone + Value> Local<'a, T> {
    pub fn upcast(&self) -> Local<'a, Any> {
        Local {
            value: Any(self.value.to_raw()),
            phantom: PhantomData
        }
    }
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
#[derive(Clone)]
pub struct Any(raw::Local);

impl Value for Any {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Any(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Any(ref mut local) = self;
        local
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Undefined(raw::Local);

impl Value for Undefined {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Undefined(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Undefined(ref mut local) = self;
        local
    }
}

impl Undefined {
    fn new<'a>() -> Local<'a, Undefined> {
        let mut result = Local {
            value: Undefined(unsafe { mem::uninitialized() }),
            phantom: PhantomData
        };
        match &mut result {
            &mut Local { value: Undefined(ref mut undefined), .. } => {
                unsafe {
                    Nan_NewUndefined(undefined);
                }
            }
        }
        result
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Null(raw::Local);

impl Value for Null {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Null(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Null(ref mut local) = self;
        local
    }
}

impl Null {
    fn new<'a>() -> Local<'a, Null> {
        let mut result = Local {
            value: Null(unsafe { mem::uninitialized() }),
            phantom: PhantomData
        };
        match &mut result {
            &mut Local { value: Null(ref mut null), .. } => {
                unsafe {
                    Nan_NewNull(null);
                }
            }
        }
        result
    }
}

#[repr(C)]
pub struct String(raw::Local);

impl Value for String {
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

impl Value for Integer {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Integer(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Integer(ref mut local) = self;
        local
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
    fn to_raw_ref(&self) -> &raw::Local {
        let &Number(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Number(ref mut local) = self;
        local
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
    fn to_raw_ref(&self) -> &raw::Local {
        let &Object(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Object(ref mut local) = self;
        local
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

impl Value for Array {
    fn to_raw_ref(&self) -> &raw::Local {
        let &Array(ref local) = self;
        local
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Array(ref mut local) = self;
        local
    }
}

// alternative and much more verbose upcast implementation:
/*
        let mut result = Local {
            value: Any(unsafe { mem::uninitialized() }),
            phantom: PhantomData
        };
        match &mut result {
            &mut Local { value: Any(ref mut any), .. } => {
                unsafe {
                    Nan_UpcastArray(any, local.to_raw());
                }
            }
        }
        result
*/

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

#[repr(C)]
pub struct Realm(raw::Isolate);

impl Realm {
    pub fn scoped<'root, T: Debug, F: FnOnce(&RootScope<'root>) -> T>(&'root self, f: F) -> T {
        let closure: Box<F> = Box::new(f);
        let callback: extern "C" fn(&mut Box<Option<T>>, &'root Realm, Box<F>) = root_callback::<'root, T, F>;
        let mut result: Box<Option<T>> = Box::new(None);
        {
            let out: &mut Box<Option<T>> = &mut result;
            unsafe {
                let out: *mut c_void = mem::transmute(out);
                let closure: *mut c_void = mem::transmute(closure);
                let callback: extern "C" fn(&mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
                let isolate: *mut c_void = mem::transmute(self);
                Nan_Root(out, closure, callback, isolate);
            }
        }
        result.unwrap()
    }
}

fn ensure_active<'root, T: Scope<'root>>(scope: &T) {
    if !scope.active() {
        panic!("illegal attempt to allocate local for inactive scope");
    }
}

pub trait Scope<'root>: Sized {
    fn realm(&self) -> &'root Realm;
    fn nested<'outer, T: Debug, F: FnOnce(&NestedScope<'root>) -> T>(&'outer self, f: F) -> T;
    fn chained<'outer, T: Debug, F: FnOnce(&ChainedScope<'root, 'outer>) -> T>(&'outer self, f: F) -> T;

    // FIXME: define this in a private subtrait?
    fn active(&self) -> bool;

    fn undefined(&self) -> Local<Undefined> {
        ensure_active(self);
        Undefined::new()
    }

    fn null(&self) -> Local<Null> {
        ensure_active(self);
        Null::new()
    }

    // FIXME: boolean

    fn integer(&self, i: i32) -> Local<Integer> {
        ensure_active(self);
        Integer::new(i)
    }

    fn number(&self, v: f64) -> Local<Number> {
        ensure_active(self);
        Number::new(v)
    }

    fn array(&self, len: u32) -> Local<Array> {
        ensure_active(self);
        Array::new(len)
    }

    fn object(&self) -> Local<Object> {
        ensure_active(self);
        Object::new()
    }
}

trait ScopeInternal<'root>: Scope<'root> {
    fn active_cell(&self) -> &RefCell<bool>;
}

pub struct RootScope<'root> {
    realm: &'root Realm,
    active: RefCell<bool>
}

pub struct NestedScope<'root> {
    realm: &'root Realm,
    active: RefCell<bool>
}

pub struct ChainedScope<'root, 'parent> {
    realm: &'root Realm,
    active: RefCell<bool>,
    v8: *mut raw::EscapableHandleScope,
    parent: PhantomData<&'parent ()>
}

impl<'root, 'parent> ChainedScope<'root, 'parent> {
    pub fn escape<'me, T: Clone + Value>(&'me self, local: Local<'me, T>) -> Local<'parent, T> {
        let result: UnsafeCell<Local<'parent, T>> = UnsafeCell::new(Local {
            value: unsafe { mem::zeroed() },
            phantom: PhantomData
        });
        unsafe {
            Nan_EscapableHandleScope_Escape((*result.get()).value.to_raw_mut_ref(), self.v8, local.to_raw());
            result.into_inner()
        }
    }
}

impl<'root> Scope<'root> for RootScope<'root> {
    fn realm(&self) -> &'root Realm { self.realm }

    fn active(&self) -> bool {
        *self.active.borrow()
    }

    fn nested<'me, T: Debug, F: FnOnce(&NestedScope<'root>) -> T>(&'me self, f: F) -> T {
        nest(self, f)
    }

    fn chained<'me, T: Debug, F: FnOnce(&ChainedScope<'root, 'me>) -> T>(&'me self, f: F) -> T {
        chain(self, f)
    }
}

extern "C" fn chained_callback<'root, 'parent, T, P, F>(out: &mut Box<Option<T>>,
                                                        parent: &'parent P,
                                                        v8: *mut raw::EscapableHandleScope,
                                                        f: Box<F>)
    where T: Debug,
          P: Scope<'root>,
          F: FnOnce(&ChainedScope<'root, 'parent>) -> T
{
    let chained = ChainedScope {
        realm: parent.realm(),
        active: RefCell::new(true),
        v8: v8,
        parent: PhantomData
    };
    let result = f(&chained);
    **out = Some(result);
}

impl<'root> ScopeInternal<'root> for RootScope<'root> {
    fn active_cell(&self) -> &RefCell<bool> {
        &self.active
    }
}

extern "C" fn root_callback<'root, T, F>(out: &mut Box<Option<T>>,
                                         realm: &'root Realm,
                                         f: Box<F>)
    where T: Debug,
          F: FnOnce(&RootScope<'root>) -> T
{
    let root = RootScope {
        realm: realm,
        active: RefCell::new(true)
    };
    let result = f(&root);
    **out = Some(result);
}

fn chain<'root, 'me, T, S, F>(outer: &'me S, f: F) -> T
    where T: Debug,
          S: ScopeInternal<'root>,
          F: FnOnce(&ChainedScope<'root, 'me>) -> T
{
    let closure: Box<F> = Box::new(f);
    let callback: extern "C" fn(&mut Box<Option<T>>, &'me S, *mut raw::EscapableHandleScope, Box<F>) = chained_callback::<'root, 'me, T, S, F>;
    let mut result: Box<Option<T>> = Box::new(None);
    {
        let out: &mut Box<Option<T>> = &mut result;
        { *outer.active_cell().borrow_mut() = false; }
        unsafe {
            let out: *mut c_void = mem::transmute(out);
            let closure: *mut c_void = mem::transmute(closure);
            let callback: extern "C" fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
            let this: *mut c_void = mem::transmute(outer);
            Nan_Chained(out, closure, callback, this);
        }
        { *outer.active_cell().borrow_mut() = true; }
    }
    result.unwrap()
}

fn nest<'root, 'me, T, S, F>(outer: &'me S, f: F) -> T
    where T: Debug,
          S: ScopeInternal<'root>,
          F: FnOnce(&NestedScope<'root>) -> T
{
    let closure: Box<F> = Box::new(f);
    let callback: extern "C" fn(&mut Box<Option<T>>, &'root Realm, Box<F>) = nested_callback::<'root, T, F>;
    let mut result: Box<Option<T>> = Box::new(None);
    {
        let out: &mut Box<Option<T>> = &mut result;
        { *outer.active_cell().borrow_mut() = false; }
        unsafe {
            let out: *mut c_void = mem::transmute(out);
            let closure: *mut c_void = mem::transmute(closure);
            let callback: extern "C" fn(&mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
            let realm: *mut c_void = mem::transmute(outer.realm());
            Nan_Nested(out, closure, callback, realm);
        }
        { *outer.active_cell().borrow_mut() = true; }
    }
    result.unwrap()
}

extern "C" fn nested_callback<'root, T, F>(out: &mut Box<Option<T>>,
                                           realm: &'root Realm,
                                           f: Box<F>)
    where T: Debug,
          F: FnOnce(&NestedScope<'root>) -> T
{
    let nested = NestedScope {
        realm: realm,
        active: RefCell::new(true)
    };
    let result = f(&nested);
    **out = Some(result);
}

impl<'root> Scope<'root> for NestedScope<'root> {
    fn realm(&self) -> &'root Realm { self.realm }

    fn active(&self) -> bool {
        *self.active.borrow()
    }

    fn nested<'me, T: Debug, F: FnOnce(&NestedScope<'root>) -> T>(&'me self, f: F) -> T {
        nest(self, f)
    }

    fn chained<'outer, T: Debug, F: FnOnce(&ChainedScope<'root, 'outer>) -> T>(&'outer self, f: F) -> T {
        chain(self, f)
    }
}

impl<'root> ScopeInternal<'root> for NestedScope<'root> {
    fn active_cell(&self) -> &RefCell<bool> {
        &self.active
    }
}

impl<'root, 'parent> Scope<'root> for ChainedScope<'root, 'parent> {
    fn realm(&self) -> &'root Realm { self.realm }

    fn active(&self) -> bool {
        *self.active.borrow()
    }

    fn nested<'me, T: Debug, F: FnOnce(&NestedScope<'root>) -> T>(&'me self, f: F) -> T {
        nest(self, f)
    }

    fn chained<'outer, T: Debug, F: FnOnce(&ChainedScope<'root, 'outer>) -> T>(&'outer self, f: F) -> T {
        chain(self, f)
    }
}

impl<'root, 'parent> ScopeInternal<'root> for ChainedScope<'root, 'parent> {
    fn active_cell(&self) -> &RefCell<bool> {
        &self.active
    }
}
