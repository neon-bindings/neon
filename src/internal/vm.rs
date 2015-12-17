use std;
use std::mem;
use std::os::raw::c_void;
use neon_sys::raw;
use neon_sys::{NeonSys_ExecFunctionBody, NeonSys_ExecModuleBody, NeonSys_Call_Data, NeonSys_Call_SetReturn, NeonSys_Call_Get, NeonSys_Call_Length, NeonSys_Object_GetIsolate, NeonSys_Call_IsConstruct, NeonSys_Call_This, NeonSys_Call_Callee};
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::value::{Value, Any, AnyInternal, Object, SomeObject, Function};
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
            let mut local: raw::Local = mem::zeroed();
            NeonSys_Call_Data(&self.info, &mut local);
            Handle::new(Value::from_raw(local))
        }
    }

    pub fn set_return<'a, 'b, T: Any>(&'a self, value: Handle<'b, T>) {
        unsafe {
            NeonSys_Call_SetReturn(&self.info, value.to_raw())
        }
    }
}

pub struct Module<'a> {
    pub exports: Handle<'a, SomeObject>,
    pub scope: &'a mut RootScope<'a>
}

impl<'a> Module<'a> {
    pub fn initialize(exports: Handle<SomeObject>, init: fn(Module) -> Result<()>) {
        let mut scope = RootScope::new(unsafe { mem::transmute(NeonSys_Object_GetIsolate(exports.to_raw())) });
        unsafe {
            let kernel: *mut c_void = mem::transmute(init);
            let callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = mem::transmute(module_body_callback);
            let exports: raw::Local = exports.to_raw();
            let scope: *mut c_void = mem::transmute(&mut scope);
            NeonSys_ExecModuleBody(kernel, callback, exports, scope);
        }
    }
}

impl<'a> Module<'a> {
    pub fn export<T: Any>(&mut self, key: &str, f: fn(Call) -> JS<T>) -> Result<()> {
        let value = try!(Function::new(self.scope, f)).upcast::<Value>();
        try!(self.exports.set(key, value));
        Ok(())
    }
}

extern "C" fn module_body_callback<'a>(body: fn(Module) -> Result<()>, exports: Handle<'a, SomeObject>, scope: &'a mut RootScope<'a>) {
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
        if unsafe { NeonSys_Call_IsConstruct(mem::transmute(self.info)) } {
            CallKind::Construct
        } else {
            CallKind::Call
        }
    }

    pub fn this<'b, T: Scope<'b>>(&self, _: &mut T) -> Handle<'b, SomeObject> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            NeonSys_Call_This(mem::transmute(self.info), &mut local);
            Handle::new(SomeObject::from_raw(local))
        }
    }

    pub fn callee<'b, T: Scope<'b>>(&self, _: &mut T) -> Handle<'b, Function> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            NeonSys_Call_Callee(mem::transmute(self.info), &mut local);
            Handle::new(Function::from_raw(local))
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
            NeonSys_Call_Length(&self.info)
        }
    }

    pub fn get<'b, T: Scope<'b>>(&self, _: &mut T, i: i32) -> Option<Handle<'b, Value>> {
        if i < 0 || i >= self.len() {
            return None;
        }
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            NeonSys_Call_Get(&self.info, i, &mut local);
            Some(Handle::new(Value::from_raw(local)))
        }
    }

    pub fn require<'b, T: Scope<'b>>(&self, _: &mut T, i: i32) -> JS<'b, Value> {
        if i < 0 || i >= self.len() {
            // FIXME: throw a type error
            return Err(Throw);
        }
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            NeonSys_Call_Get(&self.info, i, &mut local);
            Ok(Handle::new(Value::from_raw(local)))
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
        NeonSys_ExecFunctionBody(closure, callback, info, scope);
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
