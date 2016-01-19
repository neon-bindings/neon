use std::mem;
use std::marker::PhantomData;
use std::collections::HashSet;
use std::os::raw::c_void;
use neon_sys;
use neon_sys::raw;
use neon_sys::buf::Buf;
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::js::{JsValue, Value, Object, JsObject, JsFunction};
use internal::js::class::{Class, JsClass};
use internal::mem::{Handle, HandleInternal, Managed};

pub struct Throw;
pub type VmResult<T> = Result<T, Throw>;
pub type JsResult<'b, T> = VmResult<Handle<'b, T>>;

#[repr(C)]
pub struct Isolate(raw::Isolate);

#[repr(C)]
pub struct CallbackInfo {
    info: raw::FunctionCallbackInfo
}

impl CallbackInfo {
    pub fn data<'a>(&self) -> Handle<'a, JsValue> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::call::data(&self.info, &mut local);
            Handle::new(JsValue::from_raw(local))
        }
    }

    pub fn set_return<'a, 'b, T: Value>(&'a self, value: Handle<'b, T>) {
        unsafe {
            neon_sys::call::set_return(&self.info, value.to_raw())
        }
    }
}

pub struct Module<'a> {
    pub exports: Handle<'a, JsObject>,
    pub scope: &'a mut RootScope<'a>
}

impl<'a> Module<'a> {
    pub fn initialize(exports: Handle<JsObject>, init: fn(Module) -> VmResult<()>) {
        let mut scope = RootScope::new(unsafe { mem::transmute(neon_sys::object::get_isolate(exports.to_raw())) });
        unsafe {
            let kernel: *mut c_void = mem::transmute(init);
            let callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = mem::transmute(module_body_callback);
            let exports: raw::Local = exports.to_raw();
            let scope: *mut c_void = mem::transmute(&mut scope);
            neon_sys::module::exec_body(kernel, callback, exports, scope);
        }
    }
}

impl<'a> Module<'a> {
    pub fn export<T: Value>(&mut self, key: &str, f: fn(Call) -> JsResult<T>) -> VmResult<()> {
        let value = try!(JsFunction::new(self.scope, f)).upcast::<JsValue>();
        try!(self.exports.set(key, value));
        Ok(())
    }
}

extern "C" fn module_body_callback<'a>(body: fn(Module) -> VmResult<()>, exports: Handle<'a, JsObject>, scope: &'a mut RootScope<'a>) {
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

fn callback_kind(info: &CallbackInfo) -> CallKind {
    if unsafe { neon_sys::call::is_construct(mem::transmute(info)) } {
        CallKind::Construct
    } else {
        CallKind::Call
    }
}

fn callback_this<'a, T: Scope<'a>>(info: &CallbackInfo, _: &mut T) -> Handle<'a, JsObject> {
    unsafe {
        let mut local: raw::Local = mem::zeroed();
        neon_sys::call::this(mem::transmute(info), &mut local);
        Handle::new(JsObject::from_raw(local))
    }
}

fn callback_callee<'a, T: Scope<'a>>(info: &CallbackInfo, _: &mut T) -> Handle<'a, JsFunction> {
    unsafe {
        let mut local: raw::Local = mem::zeroed();
        neon_sys::call::callee(mem::transmute(info), &mut local);
        Handle::new(JsFunction::from_raw(local))
    }
}

impl<'a> Call<'a> {
    pub fn kind(&self) -> CallKind {
        callback_kind(&self.info)
    }

    pub fn this<'b, T: Scope<'b>>(&self, scope: &mut T) -> Handle<'b, JsObject> {
        callback_this(&self.info, scope)
    }

    pub fn callee<'b, T: Scope<'b>>(&self, scope: &mut T) -> Handle<'b, JsFunction> {
        callback_callee(&self.info, scope)
    }
}

pub struct ConstructorCall<'a, T: Class> {
    info: &'a CallbackInfo,
    pub scope: &'a mut RootScope<'a>,
    pub arguments: Arguments<'a>,
    phantom: PhantomData<T>
}

impl<'a, T: Class> ConstructorCall<'a, T> {
    pub fn kind(&self) -> CallKind {
        callback_kind(&self.info)
    }

    pub fn this<'b, U: Scope<'b>>(&self, scope: &mut U) -> Handle<'b, JsObject> {
        callback_this(&self.info, scope)
    }

    pub fn callee<'b, U: Scope<'b>>(&self, scope: &mut U) -> Handle<'b, JsFunction> {
        callback_callee(&self.info, scope)
    }

    pub fn class<'b, U: Scope<'b>>(&self, _: &mut U) -> Handle<'b, JsClass<T>> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::class::for_constructor(mem::transmute(&self.info), &mut local);
            Handle::new(JsClass::from_raw(local))
        }
    }
}

pub struct MethodCall<'a, T: Class> {
    info: &'a CallbackInfo,
    pub scope: &'a mut RootScope<'a>,
    pub arguments: Arguments<'a>,
    phantom: PhantomData<T>
}

impl<'a, T: Class> MethodCall<'a, T> {
    pub fn kind(&self) -> CallKind {
        callback_kind(&self.info)
    }

    pub fn this<'b, U: Scope<'b>>(&self, scope: &mut U) -> Handle<'b, JsObject> {
        callback_this(&self.info, scope)
    }

    pub fn callee<'b, U: Scope<'b>>(&self, scope: &mut U) -> Handle<'b, JsFunction> {
        callback_callee(&self.info, scope)
    }

    pub fn class<'b, U: Scope<'b>>(&self, _: &mut U) -> Handle<'b, JsClass<T>> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::class::for_method(mem::transmute(&self.info), &mut local);
            Handle::new(JsClass::from_raw(local))
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
            neon_sys::call::len(&self.info)
        }
    }

    pub fn get<'b, T: Scope<'b>>(&self, _: &mut T, i: i32) -> Option<Handle<'b, JsValue>> {
        if i < 0 || i >= self.len() {
            return None;
        }
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::call::get(&self.info, i, &mut local);
            Some(Handle::new(JsValue::from_raw(local)))
        }
    }

    pub fn require<'b, T: Scope<'b>>(&self, _: &mut T, i: i32) -> JsResult<'b, JsValue> {
        if i < 0 || i >= self.len() {
            // FIXME: throw a type error
            return Err(Throw);
        }
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::call::get(&self.info, i, &mut local);
            Ok(Handle::new(JsValue::from_raw(local)))
        }
    }
}

// FIXME: change all the instances of "body" to "kernel"

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
        neon_sys::fun::exec_body(closure, callback, info, scope);
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

pub fn exec_constructor_kernel<'a, T: Class, F>(info: &'a CallbackInfo, scope: &'a mut RootScope<'a>, f: F)
    where F: FnOnce(ConstructorCall<'a, T>)
{
    let closure: Box<F> = Box::new(f);
    let callback: extern "C" fn(&'a CallbackInfo, Box<F>, &'a mut RootScope<'a>) = constructor_kernel_callback::<'a, T, F>;
    unsafe {
        let closure: *mut c_void = mem::transmute(closure);
        let callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
        let info: &c_void = mem::transmute(info);
        let scope: *mut c_void = mem::transmute(scope);
        neon_sys::class::exec_constructor_kernel(closure, callback, info, scope);
    }
}

extern "C" fn constructor_kernel_callback<'a, T: Class, F>(info: &'a CallbackInfo, body: Box<F>, scope: &'a mut RootScope<'a>)
    where F: FnOnce(ConstructorCall<'a, T>)
{
    body(ConstructorCall {
        info: info,
        scope: scope,
        arguments: unsafe { mem::transmute(info) },
        phantom: PhantomData
    });
}



pub struct LockState {
    buffers: HashSet<usize>
}

impl LockState {
    pub fn use_buffer(&mut self, buf: &Buf) {
        let p = buf.as_ptr() as usize;
        if !self.buffers.insert(p) {
            panic!("attempt to lock heap with duplicate buffers (0x{:x})", p);
        }
    }
}

pub trait Lock {
    type Internals;

    unsafe fn expose(self, state: &mut LockState) -> Self::Internals;
}

// FIXME: make a macro to do this for tuples of many size

impl<T, U> Lock for (T, U)
    where T: Lock, U: Lock
{
    type Internals = (T::Internals, U::Internals);

    unsafe fn expose(self, state: &mut LockState) -> Self::Internals {
        (self.0.expose(state), self.1.expose(state))
    }
}

// FIXME: generalize for all Iterator types?
impl<T> Lock for Vec<T>
    where T: Lock
{
    type Internals = Vec<T::Internals>;

    unsafe fn expose(self, state: &mut LockState) -> Self::Internals {
        self.into_iter()
            .map(|x| x.expose(state))
            .collect()
    }
}

pub fn lock<T, F, U>(v: T, f: F) -> U
    where T: Lock,
          F: FnOnce(T::Internals) -> U + Send
{
    let mut state = LockState { buffers: HashSet::new() };
    let internals = unsafe { v.expose(&mut state) };
    f(internals)
}
