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
    pub fn data<'a>(&self) -> Handle<'a, Value> {
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

pub struct Module<'a> {
    pub exports: &'a mut Handle<'a, Object>,
    pub scope: &'a mut RootScope<'a>
}

impl<'a> Module<'a> {
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

impl<'a> Module<'a> {
    pub fn export<T: Copy + Tagged>(&mut self, key: &str, f: fn(Call) -> JS<T>) -> Result<()> {
        let value = try!(Function::new(self.scope, f).ok_or(Throw)).upcast();
        try!(self.exports.set(self.scope, key, value));
        Ok(())
    }
}

extern "C" fn module_body_callback<'a>(body: fn(Module) -> Result<()>, exports: &'a mut Handle<'a, Object>, scope: &'a mut RootScope<'a>) {
    let _ = body(Module {
        exports: exports,
        scope: scope
    });
}

pub struct Call<'a> {
    info: &'a CallbackInfo,
    pub scope: &'a mut RootScope<'a>,
    pub arguments: Arguments<'a>
}

pub enum CallKind {
    Construct,
    Call
}

impl<'a> Call<'a> {
    pub fn kind(&self) -> CallKind {
        if unsafe { Nan_FunctionCallbackInfo_IsConstructCall(mem::transmute(self.info)) } {
            CallKind::Construct
        } else {
            CallKind::Call
        }
    }

    pub fn this<'b, T: Scope<'b>>(&self, _: &mut T) -> Handle<'b, Object> {
        unsafe {
            let mut result = Object::zero_internal();
            Nan_FunctionCallbackInfo_This(mem::transmute(self.info), result.to_raw_mut_ref());
            result
        }
    }

    pub fn callee<'b, T: Scope<'b>>(&self, _: &mut T) -> Handle<'b, Function> {
        unsafe {
            let mut result = Function::zero_internal();
            Nan_FunctionCallbackInfo_Callee(mem::transmute(self.info), result.to_raw_mut_ref());
            result
        }
    }
}

#[repr(C)]
pub struct Arguments<'a> {
    info: &'a raw::FunctionCallbackInfo
}

impl<'a> Arguments<'a> {
    pub fn len(&self) -> i32 {
        unsafe {
            Nan_FunctionCallbackInfo_Length(&self.info)
        }
    }

    pub fn get<'b, T: Scope<'b>>(&self, _: &mut T, i: i32) -> Option<Handle<'b, Value>> {
        if i < 0 || i >= self.len() {
            return None;
        }
        unsafe {
            let mut result = Value::zero_internal();
            Nan_FunctionCallbackInfo_Get(&self.info, i, result.to_raw_mut_ref());
            Some(result)
        }
    }

    pub fn require<'b, T: Scope<'b>>(&self, _: &mut T, i: i32) -> JS<'b, Value> {
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

pub fn exec_function_body<'a, F>(info: &'a CallbackInfo, scope: &'a mut RootScope<'a>, f: F)
    where F: FnOnce(Call<'a>)
{
    let closure: Box<F> = Box::new(f);
    let callback: extern "C" fn(&'a CallbackInfo, Box<F>, &'a mut RootScope<'a>) = function_body_callback::<'a, F>;
    unsafe {
        let closure: *mut c_void = mem::transmute(closure);
        let callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
        let info: &c_void = mem::transmute(info);
        let scope: *mut c_void = mem::transmute(scope);
        Nanny_ExecFunctionBody(closure, callback, info, scope);
    }
}

extern "C" fn function_body_callback<'a, F>(info: &'a CallbackInfo, body: Box<F>, scope: &'a mut RootScope<'a>)
    where F: FnOnce(Call<'a>)
{
    body(Call {
        info: info,
        scope: scope,
        arguments: unsafe { mem::transmute(info) }
    });
}
