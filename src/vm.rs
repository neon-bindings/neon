use std::mem;
use std::os::raw::c_void;
use std::cell::{RefCell, UnsafeCell};
use nanny_sys::raw;
use nanny_sys::{Nan_FunctionCallbackInfo_SetReturnValue, Nan_FunctionCallbackInfo_GetIsolate, Nan_FunctionCallbackInfo_IsConstructCall, Nan_FunctionCallbackInfo_This, Nan_FunctionCallbackInfo_Length, Nan_FunctionCallbackInfo_Get, Nan_Root};
use internal::mem::{Handle, HandleInternal};
use internal::value::{Value, Object, ObjectInternal, Any, AnyInternal};
use internal::scope::{Scope, RootScope, RootScopeInternal};

#[repr(C)]
pub struct Call {
    activation: UnsafeCell<Activation>
}

#[repr(C)]
pub struct Activation {
    info: raw::FunctionCallbackInfo
}

#[repr(C)]
pub struct Arguments {
    info: raw::FunctionCallbackInfo
}

pub enum CallKind {
    Construct,
    Call
}

impl Call {
    fn info(&self) -> &raw::FunctionCallbackInfo {
        unsafe {
            &(*(self.activation.get())).info
        }
    }

    pub fn activation(&self) -> &mut Activation {
        unsafe {
            mem::transmute(self.activation.get())
        }
    }

    pub fn arguments(&self) -> &Arguments {
        unsafe {
            mem::transmute(self.activation.get())
        }
    }

    pub fn this<'fun, 'block, 'scope, T: Scope<'fun, 'block>>(&self, _: &'scope mut T) -> Handle<'block, Object> {
        unsafe {
            let mut result = Object::zero_internal();
            Nan_FunctionCallbackInfo_This(self.info(), result.to_raw_mut_ref());
            result
        }
    }

    pub fn kind(&self) -> CallKind {
        if unsafe { Nan_FunctionCallbackInfo_IsConstructCall(self.info()) } {
            CallKind::Construct
        } else {
            CallKind::Call
        }
    }

    pub fn realm(&self) -> &Realm {
        unsafe {
            mem::transmute(Nan_FunctionCallbackInfo_GetIsolate(self.info()))
        }
    }
}

impl Activation {
    // GC: Storing a Handle in a return value keeps it alive independent of any Scope.
    pub fn set_return<'a, 'b, T: Clone + Value>(&'a mut self, value: Handle<'b, T>) {
         unsafe {
            Nan_FunctionCallbackInfo_SetReturnValue(&mut self.info, value.to_raw());
        }
    }
}

impl Arguments {
    pub fn len(&self) -> i32 {
        unsafe {
            Nan_FunctionCallbackInfo_Length(&self.info)
        }
    }

    pub fn get<'fun, 'block, 'scope, T: Scope<'fun, 'block>>(&self, _: &'scope mut T, i: i32) -> Handle<'block, Any> {
        if i < 0 || i >= self.len() {
            panic!("arguments vector index out of range: {}", i);
        }
        unsafe {
            let mut result = Any::zero_internal();
            Nan_FunctionCallbackInfo_Get(&self.info, i, result.to_raw_mut_ref());
            result
        }
    }
}

#[repr(C)]
pub struct Realm(raw::Isolate);

impl Realm {
    pub fn scoped<'fun, T, F: for<'block> FnOnce(&mut RootScope<'fun, 'block>) -> T>(&'fun self, f: F) -> T {
        let closure: Box<F> = Box::new(f);
        let callback: extern "C" fn(&mut Box<Option<T>>, &'fun Realm, Box<F>) = root_callback::<'fun, T, F>;
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

extern "C" fn root_callback<'fun, T, F>(out: &mut Box<Option<T>>,
                                        realm: &'fun Realm,
                                        f: Box<F>)
    where F: for<'block> FnOnce(&mut RootScope<'fun, 'block>) -> T
{
    let mut root = RootScope::new(realm, RefCell::new(true));
    let result = f(&mut root);
    **out = Some(result);
}
