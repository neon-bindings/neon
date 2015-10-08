use std::fmt::Debug;
use std::mem;
use std::os::raw::c_void;
use std::cell::{RefCell, UnsafeCell};
use nanny_sys::raw;
use nanny_sys::{Nan_FunctionCallbackInfo_SetReturnValue, Nan_FunctionCallbackInfo_GetIsolate, Nan_Root};
use local::Local;
use value::Value;
use internal::scope::{RootScope, RootScopeInternal};

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

extern "C" fn root_callback<'root, T, F>(out: &mut Box<Option<T>>,
                                         realm: &'root Realm,
                                         f: Box<F>)
    where T: Debug,
          F: FnOnce(&RootScope<'root>) -> T
{
    let root = RootScope::new(realm, RefCell::new(true));
/*
    let root = RootScope {
        realm: realm,
        active: RefCell::new(true)
    };
*/
    let result = f(&root);
    **out = Some(result);
}
