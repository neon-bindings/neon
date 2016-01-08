use std::mem;
use std::collections::HashSet;
use std::os::raw::c_void;
use neon_sys;
use neon_sys::raw;
use neon_sys::buf::Buf;
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::js::{JsValue, Value, ValueInternal, Object, JsObject, JsFunction};
use internal::mem::{Handle, HandleInternal};

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

impl<'a> Call<'a> {
    pub fn kind(&self) -> CallKind {
        if unsafe { neon_sys::call::is_construct(mem::transmute(self.info)) } {
            CallKind::Construct
        } else {
            CallKind::Call
        }
    }

    pub fn this<'b, T: Scope<'b>>(&self, _: &mut T) -> Handle<'b, JsObject> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::call::this(mem::transmute(self.info), &mut local);
            Handle::new(JsObject::from_raw(local))
        }
    }

    pub fn callee<'b, T: Scope<'b>>(&self, _: &mut T) -> Handle<'b, JsFunction> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::call::callee(mem::transmute(self.info), &mut local);
            Handle::new(JsFunction::from_raw(local))
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
