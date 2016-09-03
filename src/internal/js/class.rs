use std::any::{Any, TypeId};
use std::mem;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr::null_mut;
use neon_sys;
use neon_sys::raw;
use internal::mem::{Handle, HandleInternal, Managed};
use internal::scope::{Scope, ScopeInternal, RootScopeInternal};
use internal::vm::{Isolate, IsolateInternal, JsResult, VmResult, FunctionCall, CallbackInfo, Lock, LockState, Throw, This, Kernel};
use internal::js::{Value, ValueInternal, JsFunction, JsObject, Object, JsValue, JsUndefined, build};
use internal::js::error::{JsError, Kind};

#[repr(C)]
pub struct MethodKernel<T: Class>(fn(FunctionCall<T>) -> JsResult<JsValue>);

impl<T: Class> MethodKernel<T> {
    pub fn new(kernel: fn(FunctionCall<T>) -> JsResult<JsValue>) -> Self {
        MethodKernel(kernel)
    }
}

impl<T: Class> Kernel<()> for MethodKernel<T> {
    extern "C" fn callback(info: &CallbackInfo) {
        info.scope().with(|scope| {
            let data = info.data();
            let call = info.as_call(scope);
            // Note: This is pretty sleazy, pretending to be a Handle<T> before doing the check.
            //       But we're doing the check immediately, so it seems tolerable.
            let this: Handle<T> = call.arguments.this(call.scope);
            if !this.is_a::<T>() {
                if let Ok(metadata) = T::metadata(call.scope) {
                    unsafe {
                        neon_sys::class::throw_this_error(mem::transmute(call.scope.isolate()), metadata.pointer);
                    }
                }
                return;
            }
            let MethodKernel(kernel) = unsafe { Self::from_wrapper(data.to_raw()) };
            if let Ok(value) = kernel(call) {
                info.set_return(value);
            }
        })
    }

    unsafe fn from_wrapper(h: raw::Local) -> Self {
        MethodKernel(mem::transmute(neon_sys::fun::get_kernel(h)))
    }

    fn as_ptr(self) -> *mut c_void {
        unsafe { mem::transmute(self.0) }
    }
}

#[repr(C)]
pub struct ConstructorCallKernel(fn(FunctionCall<JsValue>) -> JsResult<JsValue>);

impl ConstructorCallKernel {
    pub fn new(kernel: fn(FunctionCall<JsValue>) -> JsResult<JsValue>) -> Self {
        ConstructorCallKernel(kernel)
    }

    extern "C" fn unimplemented<T: Class>(info: &CallbackInfo) {
        let mut scope = info.scope();
        if let Ok(metadata) = T::metadata(&mut scope) {
            unsafe {
                neon_sys::class::throw_call_error(mem::transmute(scope.isolate()), metadata.pointer);
            }
        }
        return;
    }
}

impl Kernel<()> for ConstructorCallKernel {
    extern "C" fn callback(info: &CallbackInfo) {
        info.scope().with(|scope| {
            let data = info.data();
            let ConstructorCallKernel(kernel) = unsafe { Self::from_wrapper(data.to_raw()) };
            let call = info.as_call(scope);
            if let Ok(value) = kernel(call) {
                info.set_return(value);
            }
        })
    }

    unsafe fn from_wrapper(h: raw::Local) -> Self {
        ConstructorCallKernel(mem::transmute(neon_sys::class::get_call_kernel(h)))
    }

    fn as_ptr(self) -> *mut c_void {
        unsafe { mem::transmute(self.0) }
    }
}

#[repr(C)]
pub struct AllocateKernel<T: Class>(fn(FunctionCall<JsUndefined>) -> VmResult<T::Internals>);

impl<T: Class> AllocateKernel<T> {
    pub fn new(kernel: fn(FunctionCall<JsUndefined>) -> VmResult<T::Internals>) -> Self {
        AllocateKernel(kernel)
    }
}

impl<T: Class> Kernel<*mut c_void> for AllocateKernel<T> {
    extern "C" fn callback(info: &CallbackInfo) -> *mut c_void {
        info.scope().with(|scope| {
            let data = info.data();
            let AllocateKernel(kernel) = unsafe { Self::from_wrapper(data.to_raw()) };
            let call = info.as_call(scope);
            if let Ok(value) = kernel(call) {
                let p = Box::into_raw(Box::new(value));
                unsafe { mem::transmute(p) }
            } else {
                null_mut()
            }
        })
    }

    unsafe fn from_wrapper(h: raw::Local) -> Self {
        AllocateKernel(mem::transmute(neon_sys::class::get_allocate_kernel(h)))
    }

    fn as_ptr(self) -> *mut c_void {
        unsafe { mem::transmute(self.0) }
    }
}

#[repr(C)]
pub struct ConstructKernel<T: Class>(fn(FunctionCall<T>) -> VmResult<Option<Handle<JsObject>>>);

impl<T: Class> ConstructKernel<T> {
    pub fn new(kernel: fn(FunctionCall<T>) -> VmResult<Option<Handle<JsObject>>>) -> Self {
        ConstructKernel(kernel)
    }
}

impl<T: Class> Kernel<bool> for ConstructKernel<T> {
    extern "C" fn callback(info: &CallbackInfo) -> bool {
        info.scope().with(|scope| {
            let data = info.data();
            let ConstructKernel(kernel) = unsafe { Self::from_wrapper(data.to_raw()) };
            let call = info.as_call(scope);
            match kernel(call) {
                Ok(None) => true,
                Ok(Some(obj)) => {
                    info.set_return(obj);
                    true
                }
                _ => false
            }
        })
    }

    unsafe fn from_wrapper(h: raw::Local) -> Self {
        ConstructKernel(mem::transmute(neon_sys::class::get_construct_kernel(h)))
    }

    fn as_ptr(self) -> *mut c_void {
        unsafe { mem::transmute(self.0) }
    }
}

pub struct ClassDescriptor<'a, T: Class> {
    name: &'a str,
    allocate: AllocateKernel<T>,
    call: Option<ConstructorCallKernel>,
    construct: Option<ConstructKernel<T>>,
    methods: Vec<(&'a str, MethodKernel<T>)>
}

impl<'a, T: Class> ClassDescriptor<'a, T> {
    pub fn new<'b, U: Class>(name: &'b str, allocate: AllocateKernel<U>) -> ClassDescriptor<'b, U> {
        ClassDescriptor {
            name: name,
            allocate: allocate,
            call: None,
            construct: None,
            methods: Vec::new()
        }
    }

    pub fn call(mut self, kernel: ConstructorCallKernel) -> Self {
        self.call = Some(kernel);
        self
    }

    pub fn construct(mut self, kernel: ConstructKernel<T>) -> Self {
        self.construct = Some(kernel);
        self
    }

    pub fn method(mut self, name: &'a str, kernel: MethodKernel<T>) -> Self {
        self.methods.push((name, kernel));
        self
    }
}

extern "C" fn drop_internals<T>(internals: *mut c_void) {
    let p: Box<T> = unsafe { Box::from_raw(mem::transmute(internals)) };
    mem::drop(p);
}

pub trait Class: Managed + Any {
    type Internals;

    fn setup<'a, T: Scope<'a>>(_: &mut T) -> VmResult<ClassDescriptor<'a, Self>>;

    fn class<'a, T: Scope<'a>>(scope: &mut T) -> JsResult<'a, JsClass<Self>> {
        let metadata = try!(Self::metadata(scope));
        Ok(unsafe { metadata.class(scope) })
    }

    fn describe<'a>(name: &'a str, allocate: AllocateKernel<Self>) -> ClassDescriptor<'a, Self> {
        ClassDescriptor::<Self>::new(name, allocate)
    }
}

unsafe impl<T: Class> This for T {
    fn as_this(h: raw::Local) -> Self {
        Self::from_raw(h)
    }
}

impl<T: Class> Object for T { }

pub trait ClassInternal: Class {
    fn metadata_opt<'a, T: Scope<'a>>(scope: &mut T) -> Option<ClassMetadata> {
        scope.isolate()
             .class_map()
             .get(&TypeId::of::<Self>())
             .map(|m| m.clone())
    }

    fn metadata<'a, T: Scope<'a>>(scope: &mut T) -> VmResult<ClassMetadata> {
        match Self::metadata_opt(scope) {
            Some(metadata) => Ok(metadata),
            None => Self::create(scope)
        }
    }

    fn create<'a, T: Scope<'a>>(scope: &mut T) -> VmResult<ClassMetadata> {
        let descriptor = try!(Self::setup(scope));
        unsafe {
            let isolate: *mut c_void = mem::transmute(scope.isolate());

            let (allocate_callback, allocate_kernel) = descriptor.allocate.export();

            let (construct_callback, construct_kernel) = match descriptor.construct {
                Some(k) => k.export(),
                None    => (null_mut(), null_mut())
            };

            let (call_callback, call_kernel) = match descriptor.call {
                Some(k) => k.export(),
                None    => (mem::transmute(ConstructorCallKernel::unimplemented::<Self> as *mut c_void), null_mut())
            };

            let metadata_pointer = neon_sys::class::create_base(isolate,
                                                                allocate_callback, allocate_kernel,
                                                                construct_callback, construct_kernel,
                                                                call_callback, call_kernel,
                                                                drop_internals::<Self::Internals>);

            if metadata_pointer.is_null() {
                return Err(Throw);
            }

            // NOTE: None of the error cases below need to delete the ClassMetadata object, since the
            //       v8::FunctionTemplate has a finalizer that will delete it.

            let class_name = descriptor.name;
            if !neon_sys::class::set_name(isolate, metadata_pointer, class_name.as_ptr(), class_name.len() as u32) {
                return Err(Throw);
            }

            for (name, method) in descriptor.methods {
                let method: Handle<JsFunction> = try!(build(|out| {
                    let (method_callback, method_kernel) = method.export();
                    neon_sys::fun::new(out, isolate, method_callback, method_kernel)
                }));
                if !neon_sys::class::add_method(isolate, metadata_pointer, name.as_ptr(), name.len() as u32, method.to_raw()) {
                    return Err(Throw);
                }
            }

            let metadata = ClassMetadata {
                pointer: metadata_pointer
            };

            scope.isolate().class_map().set(TypeId::of::<Self>(), metadata);

            Ok(metadata)
        }
    }
}

impl<T: Class> ClassInternal for T { }

impl<T: Class> ValueInternal for T {
    fn is_typeof<Other: Value>(value: Other) -> bool {
        let mut isolate: Isolate = unsafe {
            mem::transmute(neon_sys::call::current_isolate())
        };
        let map = isolate.class_map();
        match map.get(&TypeId::of::<T>()) {
            None => false,
            Some(ref metadata) => unsafe {
                metadata.has_instance(value.to_raw())
            }
        }
    }
}

impl<T: Class> Value for T { }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsClass<T: Class> {
    handle: raw::Local,
    phantom: PhantomData<T>
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassMetadata {
    pointer: *mut c_void
}

impl ClassMetadata {
    pub unsafe fn class<'a, T: Class, U: Scope<'a>>(&self, scope: &mut U) -> Handle<'a, JsClass<T>> {
        let mut local: raw::Local = mem::zeroed();
        neon_sys::class::metadata_to_class(&mut local, mem::transmute(scope.isolate()), self.pointer);
        Handle::new(JsClass {
            handle: local,
            phantom: PhantomData
        })
    }

    pub unsafe fn has_instance(&self, value: raw::Local) -> bool {
        neon_sys::class::has_instance(self.pointer, value)
    }
}

impl<T: Class> JsClass<T> {
    pub fn check<U: Value>(&self, v: Handle<U>, msg: &str) -> JsResult<T> {
        let local = v.to_raw();
        if unsafe { neon_sys::class::check(self.to_raw(), local) } {
            Ok(Handle::new(T::from_raw(local)))
        } else {
            JsError::throw(Kind::TypeError, msg)
        }
    }

    pub fn constructor<'a, U: Scope<'a>>(&self, _: &mut U) -> JsResult<'a, JsFunction<T>> {
        build(|out| {
            unsafe {
                neon_sys::class::constructor(out, self.to_raw())
            }
        })
    }
}

impl<'a, T: Class> Lock for &'a mut T {
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
