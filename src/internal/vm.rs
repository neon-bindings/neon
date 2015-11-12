use std;
use std::mem;
use std::os::raw::c_void;
use nanny_sys::raw;
use nanny_sys::{Nanny_ExecFunctionBody, Nanny_ExecModuleBody, Nan_FunctionCallbackInfo_Data, Nan_FunctionCallbackInfo_SetReturnValue, Nan_FunctionCallbackInfo_Get, Nan_FunctionCallbackInfo_Length, Nan_Object_GetIsolate, Nan_FunctionCallbackInfo_IsConstructCall, Nan_FunctionCallbackInfo_This, Nan_FunctionCallbackInfo_Callee};
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::value::{Value, ValueInternal, Tagged, TaggedInternal, Object, ObjectInternal, Function, FunctionInternal};
use internal::mem::{Handle, HandleInternal};

pub struct Throw;
pub type Result<T> = std::result::Result<T, Throw>;
pub type JS<'b, T> = Result<Handle<'b, T>>;

#[repr(C)]
pub struct Isolate(raw::Isolate);

#[repr(C)]
pub struct CallbackInfo {
    info: raw::FunctionCallbackInfo
}

impl CallbackInfo {
    pub fn data<'block>(&self) -> Handle<'block, Value> {
        unsafe {
            let mut result = Value::zero_internal();
            Nan_FunctionCallbackInfo_Data(&self.info, result.to_raw_mut_ref());
            result
        }
    }

    pub fn set_return<'a, 'b, T: Copy + Tagged>(&'a self, value: Handle<'b, T>) {
        unsafe {
            Nan_FunctionCallbackInfo_SetReturnValue(&self.info, value.to_raw())
        }
    }
}

pub struct Module<'top> {
    pub exports: &'top mut Handle<'top, Object>,
    pub scope: &'top mut RootScope<'top, 'top>
}

impl<'top> Module<'top> {
    pub fn initialize(exports: &mut Handle<Object>, init: fn(Module) -> Result<()>) {
        let mut scope = RootScope::new(unsafe { mem::transmute(Nan_Object_GetIsolate(exports.to_raw_ref())) });
        unsafe {
            let kernel: *mut c_void = mem::transmute(init);
            let callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = mem::transmute(module_body_callback);
            let exports: &mut raw::Local = exports.to_raw_mut_ref();
            let scope: *mut c_void = mem::transmute(&mut scope);
            Nanny_ExecModuleBody(kernel, callback, exports, scope);
        }
    }
}

impl<'top> Module<'top> {
    pub fn export<T: Copy + Tagged>(&mut self, key: &str, f: fn(Call) -> JS<T>) -> Result<()> {
        let value = try!(Function::new(self.scope, f).ok_or(Throw)).upcast();
        try!(self.exports.set(self.scope, key, value));
        Ok(())
    }
}

extern "C" fn module_body_callback<'top>(body: fn(Module) -> Result<()>, exports: &'top mut Handle<'top, Object>, scope: &'top mut RootScope<'top, 'top>) {
    let _ = body(Module {
        exports: exports,
        scope: scope
    });
}

pub struct Call<'fun> {
    info: &'fun CallbackInfo,
    pub scope: &'fun mut RootScope<'fun, 'fun>,
    pub arguments: Arguments<'fun>
}

pub enum CallKind {
    Construct,
    Call
}

impl<'fun> Call<'fun> {
    pub fn kind(&self) -> CallKind {
        if unsafe { Nan_FunctionCallbackInfo_IsConstructCall(mem::transmute(self.info)) } {
            CallKind::Construct
        } else {
            CallKind::Call
        }
    }

    pub fn this<'block, 'scope, T: Scope<'fun, 'block>>(&self, _: &'scope mut T) -> Handle<'block, Object> {
        unsafe {
            let mut result = Object::zero_internal();
            Nan_FunctionCallbackInfo_This(mem::transmute(self.info), result.to_raw_mut_ref());
            result
        }
    }

    pub fn callee<'block, 'scope, T: Scope<'fun, 'block>>(&self, _: &'scope mut T) -> Handle<'block, Function> {
        unsafe {
            let mut result = Function::zero_internal();
            Nan_FunctionCallbackInfo_Callee(mem::transmute(self.info), result.to_raw_mut_ref());
            result
        }
    }
}

#[repr(C)]
pub struct Arguments<'fun> {
    info: &'fun raw::FunctionCallbackInfo
}

impl<'fun> Arguments<'fun> {
    pub fn len(&self) -> i32 {
        unsafe {
            Nan_FunctionCallbackInfo_Length(&self.info)
        }
    }

    pub fn get<'block, 'scope, T: Scope<'fun, 'block>>(&self, _: &'scope mut T, i: i32) -> Option<Handle<'block, Value>> {
        if i < 0 || i >= self.len() {
            return None;
        }
        unsafe {
            let mut result = Value::zero_internal();
            Nan_FunctionCallbackInfo_Get(&self.info, i, result.to_raw_mut_ref());
            Some(result)
        }
    }

    pub fn require<'block, 'scope, T: Scope<'fun, 'block>>(&self, _: &'scope mut T, i: i32) -> JS<'block, Value> {
        if i < 0 || i >= self.len() {
            // FIXME: throw a type error
            return Err(Throw);
        }
        unsafe {
            let mut result = Value::zero_internal();
            Nan_FunctionCallbackInfo_Get(&self.info, i, result.to_raw_mut_ref());
            Ok(result)
        }
    }
}

pub fn exec_function_body<'fun, F>(info: &'fun CallbackInfo, scope: &'fun mut RootScope<'fun, 'fun>, f: F)
    where F: FnOnce(Call<'fun>)
{
    let closure: Box<F> = Box::new(f);
    let callback: extern "C" fn(&'fun CallbackInfo, Box<F>, &'fun mut RootScope<'fun, 'fun>) = function_body_callback::<'fun, F>;
    unsafe {
        let closure: *mut c_void = mem::transmute(closure);
        let callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
        let info: &c_void = mem::transmute(info);
        let scope: *mut c_void = mem::transmute(scope);
        Nanny_ExecFunctionBody(closure, callback, info, scope);
    }
}

extern "C" fn function_body_callback<'fun, F>(info: &'fun CallbackInfo, body: Box<F>, scope: &'fun mut RootScope<'fun, 'fun>)
    where F: FnOnce(Call<'fun>)
{
    body(Call {
        info: info,
        scope: scope,
        arguments: unsafe { mem::transmute(info) }
    });
}
