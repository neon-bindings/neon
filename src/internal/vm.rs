use std::mem;
use std::any::TypeId;
use std::marker::PhantomData;
use std::collections::{HashSet, HashMap};
use std::os::raw::c_void;
use cslice::CMutSlice;
use neon_sys;
use neon_sys::raw;
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::js::{JsValue, Value, Object, JsObject, JsFunction};
use internal::js::class::ClassMetadata;
use internal::js::error::JsTypeError;
use internal::mem::{Handle, HandleInternal, Managed};

pub struct Throw;
pub type VmResult<T> = Result<T, Throw>;
pub type JsResult<'b, T> = VmResult<Handle<'b, T>>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Isolate(*mut raw::Isolate);

pub trait IsolateInternal {
    fn to_raw(self) -> *mut raw::Isolate;
    fn from_raw(ptr: *mut raw::Isolate) -> Self;
    fn class_map(&mut self) -> &mut ClassMap;
}

pub struct ClassMap {
    map: HashMap<TypeId, ClassMetadata>
}

impl ClassMap {
    fn new() -> ClassMap {
        ClassMap {
            map: HashMap::new()
        }
    }

    pub fn get(&self, key: &TypeId) -> Option<&ClassMetadata> {
        self.map.get(key)
    }

    pub fn set(&mut self, key: TypeId, val: ClassMetadata) {
        self.map.insert(key, val);
    }
}

impl IsolateInternal for Isolate {
    fn to_raw(self) -> *mut raw::Isolate {
        let Isolate(ptr) = self;
        ptr
    }

    fn from_raw(ptr: *mut raw::Isolate) -> Self {
        Isolate(ptr)
    }

    fn class_map(&mut self) -> &mut ClassMap {
        let mut ptr: *mut c_void = unsafe { neon_sys::class::get_class_map(self.to_raw()) };
        if ptr.is_null() {
            let b: Box<ClassMap> = Box::new(ClassMap::new());
            let raw = Box::into_raw(b);
            ptr = unsafe { mem::transmute(raw) };
            let free_map: *mut c_void = unsafe { mem::transmute(drop_class_map) };
            unsafe {
                neon_sys::class::set_class_map(self.to_raw(), ptr, free_map);
            }
        }
        unsafe { mem::transmute(ptr) }
    }
}

extern "C" fn drop_class_map(map: Box<ClassMap>) {
    mem::drop(map);
}


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

    pub fn scope(&self) -> RootScope {
        RootScope::new(unsafe {
            mem::transmute(neon_sys::call::get_isolate(mem::transmute(self)))
        })
    }

    pub fn set_return<'a, 'b, T: Value>(&'a self, value: Handle<'b, T>) {
        unsafe {
            neon_sys::call::set_return(&self.info, value.to_raw())
        }
    }

    pub fn as_call<'a, T: This>(&'a self, scope: &'a mut RootScope<'a>) -> FunctionCall<'a, T> {
        FunctionCall {
            info: self,
            scope: scope,
            arguments: Arguments {
                info: &self,
                phantom: PhantomData
            }
        }
    }

    fn kind(&self) -> CallKind {
        if unsafe { neon_sys::call::is_construct(mem::transmute(self)) } {
            CallKind::Construct
        } else {
            CallKind::Call
        }
    }

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
            return JsTypeError::throw("not enough arguments");
        }
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::call::get(&self.info, i, &mut local);
            Ok(Handle::new(JsValue::from_raw(local)))
        }
    }

    pub fn this<'b, T: Scope<'b>>(&self, _: &mut T) -> raw::Local {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::call::this(mem::transmute(&self.info), &mut local);
            local
        }
    }

    pub fn callee<'a, T: Scope<'a>>(&self, _: &mut T) -> Handle<'a, JsFunction> {
        unsafe {
            let mut local: raw::Local = mem::zeroed();
            neon_sys::call::callee(mem::transmute(&self.info), &mut local);
            Handle::new(JsFunction::from_raw(local))
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
            let callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = mem::transmute(module_callback);
            let exports: raw::Local = exports.to_raw();
            let scope: *mut c_void = mem::transmute(&mut scope);
            neon_sys::module::exec_kernel(kernel, callback, exports, scope);
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

extern "C" fn module_callback<'a>(kernel: fn(Module) -> VmResult<()>, exports: Handle<'a, JsObject>, scope: &'a mut RootScope<'a>) {
    let _ = kernel(Module {
        exports: exports,
        scope: scope
    });
}

pub trait This: Managed {
    fn as_this(h: raw::Local) -> Self;
}

pub struct FunctionCall<'a, T: This> {
    info: &'a CallbackInfo,
    pub scope: &'a mut RootScope<'a>,
    pub arguments: Arguments<'a, T>
}

pub type Call<'a> = FunctionCall<'a, JsObject>;

#[derive(Clone, Copy, Debug)]
pub enum CallKind {
    Construct,
    Call
}

impl<'a, T: This> FunctionCall<'a, T> {
    pub fn kind(&self) -> CallKind { self.info.kind() }
}

#[repr(C)]
pub struct Arguments<'a, T> {
    info: &'a CallbackInfo,
    phantom: PhantomData<T>
}

impl<'a, T: This> Arguments<'a, T> {
    pub fn len(&self) -> i32 { self.info.len() }

    pub fn get<'b, U: Scope<'b>>(&self, scope: &mut U, i: i32) -> Option<Handle<'b, JsValue>> {
        self.info.get(scope, i)
    }

    pub fn require<'b, U: Scope<'b>>(&self, scope: &mut U, i: i32) -> JsResult<'b, JsValue> {
        self.info.require(scope, i)
    }

    pub fn this<'b, U: Scope<'b>>(&self, scope: &mut U) -> Handle<'b, T> {
        Handle::new(T::as_this(self.info.this(scope)))
    }

    pub fn callee<'b, U: Scope<'b>>(&self, scope: &mut U) -> Handle<'b, JsFunction> {
        self.info.callee(scope)
    }
}

pub trait Kernel<T: Clone + Copy + Sized>: Sized {
    extern "C" fn callback(info: &CallbackInfo) -> T;

    unsafe fn from_raw(raw::Local) -> Self;

    fn as_raw(self) -> *mut c_void;

    fn pair(self) -> (*mut c_void, *mut c_void) {
        unsafe {
            (mem::transmute(Self::callback), self.as_raw())
        }
    }
}

pub fn exec_function_kernel<'a, F, T: This>(info: &'a CallbackInfo, scope: &'a mut RootScope<'a>, f: F)
    where F: FnOnce(FunctionCall<'a, T>)
{
    let closure: Box<F> = Box::new(f);
    let callback: extern "C" fn(&'a CallbackInfo, Box<F>, &'a mut RootScope<'a>) = function_callback::<'a, F, T>;
    unsafe {
        let closure: *mut c_void = mem::transmute(closure);
        let callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
        let info: &c_void = mem::transmute(info);
        let scope: *mut c_void = mem::transmute(scope);
        neon_sys::fun::exec_kernel(closure, callback, info, scope);
    }
}

extern "C" fn function_callback<'a, F, T: This>(info: &'a CallbackInfo, kernel: Box<F>, scope: &'a mut RootScope<'a>)
    where F: FnOnce(FunctionCall<'a, T>)
{
    kernel(info.as_call(scope));
}

pub struct LockState {
    buffers: HashSet<usize>
}

impl LockState {
    pub fn use_buffer(&mut self, buf: CMutSlice<u8>) {
        let p = buf.as_ptr() as usize;
        if !self.buffers.insert(p) {
            panic!("attempt to lock heap with duplicate buffers (0x{:x})", p);
        }
    }
}

pub trait Lock: Sized {
    type Internals;

    fn grab<F, T>(self, f: F) -> T
        where F: FnOnce(Self::Internals) -> T + Send
    {
        let mut state = LockState { buffers: HashSet::new() };
        let internals = unsafe { self.expose(&mut state) };
        f(internals)
    }

    unsafe fn expose(self, state: &mut LockState) -> Self::Internals;
}

// TODO: make a macro to do this for tuples of many size

impl<T, U> Lock for (T, U)
    where T: Lock, U: Lock
{
    type Internals = (T::Internals, U::Internals);

    unsafe fn expose(self, state: &mut LockState) -> Self::Internals {
        (self.0.expose(state), self.1.expose(state))
    }
}

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
