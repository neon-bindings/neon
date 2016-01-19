use std::mem;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr::null_mut;
use neon_sys;
use neon_sys::raw;
use internal::mem::{Handle, HandleInternal, Managed};
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::vm::{JsResult, ConstructorCall, MethodCall, CallbackInfo, Lock, LockState, exec_constructor_kernel};
use internal::js::{Value, JsFunction, build};
use internal::js::error::JsTypeError;

pub type Constructor<T> = fn(ConstructorCall<T>) -> JsResult<T>;
pub type Method<T> = fn(MethodCall<T>) -> JsResult<T>;

pub struct ClassDescriptor<'a, T: Class> {
    name: Option<&'a str>,
    call: Option<Constructor<T>>,
    construct: Option<Constructor<T>>,
    methods: Vec<(&'a str, Method<T>)>
}

impl<'a, T: Class> ClassDescriptor<'a, T> {
    pub fn name(&mut self, name: &'a str) -> &mut ClassDescriptor<'a, T> {
        self.name = Some(name);
        self
    }

    pub fn call(&mut self, body: Constructor<T>) -> &mut ClassDescriptor<'a, T> {
        self.call = Some(body);
        self
    }

    pub fn construct(&mut self, body: Constructor<T>) -> &mut ClassDescriptor<'a, T> {
        self.construct = Some(body);
        self
    }

    pub fn constructor(&mut self, body: Constructor<T>) -> &mut ClassDescriptor<'a, T> {
        self.call = Some(body);
        self.construct = Some(body);
        self
    }

    pub fn method(&mut self, name: &'a str, body: Method<T>) -> &mut ClassDescriptor<'a, T> {
        self.methods.push((name, body));
        self
    }

    pub fn create<'b, U: Scope<'b>>(self, scope: &mut U) -> JsResult<'b, JsClass<T>> {
        build(|out| {
            unsafe {
                let isolate: *mut c_void = mem::transmute(scope.isolate());
                let callback: extern "C" fn(&CallbackInfo) = call_neon_constructor::<T>;
                let callback: *mut c_void = mem::transmute(callback);
                let construct_kernel: *mut c_void = match self.construct {
                    Some(f) => mem::transmute(f),
                    None => null_mut()
                };
                // FIXME: need a v8try! macro for this pattern
                if !neon_sys::class::create(out, isolate, callback, construct_kernel) {
                    return false;
                }
                // FIXME: implement these
                /*
                if let Some(f) = self.call {
                    if !neon_sys::class::set_call_handler(*out, isolate, callback, mem::transmute(f)) {
                        return false;
                    }
                }
                if let Some(name) = self.name {
                    if !neon_sys::class::set_name(out, name) {
                        return false;
                    }
                }
                for (name, method) in methods {
                    // ...
                }
                 */
                // FIXME: neon_sys::class::finish(*out) to force the instantiation and disallow further modifications
                true
            }
        })
    }
}

extern "C" fn call_neon_constructor<T: Class>(info: &CallbackInfo) {
    let mut scope = RootScope::new(unsafe { mem::transmute(neon_sys::call::get_isolate(mem::transmute(info))) });
    exec_constructor_kernel(info, &mut scope, |call| {
        let data = info.data();
        let kernel: Constructor<T> = unsafe { mem::transmute(neon_sys::class::get_constructor_kernel(data.to_raw())) };
        if let Ok(value) = kernel(call) {
            info.set_return(value);
        }
    });
}

pub trait Class: Value {
    type Internals;

    fn class<'a>() -> ClassDescriptor<'a, Self> {
        ClassDescriptor {
            name: None,
            call: None,
            construct: None,
            methods: Vec::new()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsClass<T: Class> {
    handle: raw::Local,
    phantom: PhantomData<T>
}

impl<T: Class> JsClass<T> {
    pub fn check<U: Value>(&self, v: Handle<U>) -> JsResult<T> {
        let local = v.to_raw();
        if unsafe { neon_sys::class::check(self.to_raw(), local) } {
            Ok(Handle::new(T::from_raw(local)))
        } else {
            // FIXME: custom error message based on class name
            JsTypeError::throw("failed class check")
        }
    }

    pub fn new<'a, U: Scope<'a>>(&self, _: &mut U, internals: T::Internals) -> JsResult<'a, T> {
        unimplemented!()
    }

    pub fn constructor<'a, U: Scope<'a>>(&self, _: &mut U) -> JsResult<'a, JsFunction> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::class::constructor(&mut local, self.to_raw());
            Ok(Handle::new(JsFunction::from_raw(local)))
        }
    }
}

// FIXME: I believe this is unsafe. I think the Lock API needs to
// tighten the lifetime of the exposed internals not to outlive the
// lock.
impl<'a, T: Class> Lock for Handle<'a, T> {
    type Internals = &'a mut T::Internals;

    unsafe fn expose(self, _: &mut LockState) -> Self::Internals {
        let ptr: *mut c_void = neon_sys::class::get_instance_internals(self.to_raw());
        mem::transmute(ptr)
    }
}

impl<T: Class> Managed for JsClass<T> {
    fn to_raw(self) -> raw::Local { self.handle }

    fn from_raw(h: raw::Local) -> Self {
        JsClass {
            handle: h,
            phantom: PhantomData
        }
    }
}
