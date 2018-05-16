use std::any::{Any, TypeId};
use std::mem;
use std::marker::PhantomData;
use std::os::raw::c_void;
use neon_runtime;
use neon_runtime::raw;
use mem::{Handle, Managed};
use vm::{JsResult, VmResult, Throw, This, Vm, VmGuard, Ref, RefMut, LoanError, Callback};
use vm::internal::Isolate;
use js::{Value, JsFunction, Object, JsValue, Borrow, BorrowMut, build};
use js::internal::ValueInternal;
use js::error::{JsError, Kind};
use self::internal::{ClassMetadata, MethodCallback, ConstructorCallCallback, AllocateCallback, ConstructCallback};

pub(crate) mod internal {
    use std::mem;
    use std::marker::PhantomData;
    use std::os::raw::c_void;
    use std::ptr::null_mut;
    use neon_runtime;
    use neon_runtime::raw;
    use super::{JsClass, Class, ClassInternal};
    use js::{JsValue, JsObject, JsUndefined};
    use vm::{JsResult, VmResult, CallbackInfo, Callback, CallContext, Vm, Throw};
    use vm::internal::VmInternal;
    use mem::{Handle, Managed};
    use js::error::convert_panics;

    #[repr(C)]
    pub struct MethodCallback<T: Class>(pub fn(CallContext<T>) -> JsResult<JsValue>);

    impl<T: Class> Callback<()> for MethodCallback<T> {
        extern "C" fn invoke(info: &CallbackInfo) {
            unsafe {
                info.with_vm::<T, _, _>(|mut vm| {
                    let data = info.data();
                    let this: Handle<JsValue> = Handle::new_internal(JsValue::from_raw(info.this(&mut vm)));
                    if !this.is_a::<T>() {
                        if let Ok(metadata) = T::metadata(&mut vm) {
                            neon_runtime::class::throw_this_error(mem::transmute(vm.isolate()), metadata.pointer);
                        }
                        return;
                    };
                    let dynamic_callback: fn(CallContext<T>) -> JsResult<JsValue> =
                        mem::transmute(neon_runtime::fun::get_dynamic_callback(data.to_raw()));
                    if let Ok(value) = convert_panics(|| { dynamic_callback(vm) }) {
                        info.set_return(value);
                    }
                })
            }
         }

        fn as_ptr(self) -> *mut c_void {
            self.0 as *mut c_void
        }
    }

    #[repr(C)]
    pub struct ConstructorCallCallback(pub fn(CallContext<JsValue>) -> JsResult<JsValue>);

    impl ConstructorCallCallback {
        pub(crate) fn default<T: Class>() -> Self {
            fn callback<T: Class>(mut vm: CallContext<JsValue>) -> JsResult<JsValue> {
                unsafe {
                    if let Ok(metadata) = T::metadata(&mut vm) {
                        neon_runtime::class::throw_call_error(mem::transmute(vm.isolate()), metadata.pointer);
                    }
                }
                Err(Throw)
            }

            ConstructorCallCallback(callback::<T>)
        }
   }

    impl Callback<()> for ConstructorCallCallback {
        extern "C" fn invoke(info: &CallbackInfo) {
            unsafe {
                info.with_vm(|vm| {
                    let data = info.data();
                    let kernel: fn(CallContext<JsValue>) -> JsResult<JsValue> =
                        mem::transmute(neon_runtime::class::get_call_kernel(data.to_raw()));
                    if let Ok(value) = convert_panics(|| { kernel(vm) }) {
                        info.set_return(value);
                    }
                })
            }
        }

        fn as_ptr(self) -> *mut c_void {
            self.0 as *mut c_void
        }
    }

    #[repr(C)]
    pub struct AllocateCallback<T: Class>(pub fn(CallContext<JsUndefined>) -> VmResult<T::Internals>);

    impl<T: Class> Callback<*mut c_void> for AllocateCallback<T> {
        extern "C" fn invoke(info: &CallbackInfo) -> *mut c_void {
            unsafe {
                info.with_vm(|vm| {
                    let data = info.data();
                    let kernel: fn(CallContext<JsUndefined>) -> VmResult<T::Internals> =
                        mem::transmute(neon_runtime::class::get_allocate_kernel(data.to_raw()));
                    if let Ok(value) = convert_panics(|| { kernel(vm) }) {
                        let p = Box::into_raw(Box::new(value));
                        mem::transmute(p)
                    } else {
                        null_mut()
                    }
                })
            }
        }

        fn as_ptr(self) -> *mut c_void {
            self.0 as *mut c_void
        }
    }

    #[repr(C)]
    pub struct ConstructCallback<T: Class>(pub fn(CallContext<T>) -> VmResult<Option<Handle<JsObject>>>);

    impl<T: Class> Callback<bool> for ConstructCallback<T> {
        extern "C" fn invoke(info: &CallbackInfo) -> bool {
            unsafe {
                info.with_vm(|vm| {
                    let data = info.data();
                    let kernel: fn(CallContext<T>) -> VmResult<Option<Handle<JsObject>>> =
                        mem::transmute(neon_runtime::class::get_construct_kernel(data.to_raw()));
                    match convert_panics(|| { kernel(vm) }) {
                        Ok(None) => true,
                        Ok(Some(obj)) => {
                            info.set_return(obj);
                            true
                        }
                        _ => false
                    }
                })
            }
        }

        fn as_ptr(self) -> *mut c_void {
            self.0 as *mut c_void
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct ClassMetadata {
        pub(crate) pointer: *mut c_void
    }

    impl ClassMetadata {
        pub unsafe fn class<'a, T: Class, V: Vm<'a>>(&self, vm: &mut V) -> Handle<'a, JsClass<T>> {
            let mut local: raw::Local = mem::zeroed();
            neon_runtime::class::metadata_to_class(&mut local, mem::transmute(vm.isolate()), self.pointer);
            Handle::new_internal(JsClass {
                handle: local,
                phantom: PhantomData
            })
        }

        pub unsafe fn has_instance(&self, value: raw::Local) -> bool {
            neon_runtime::class::has_instance(self.pointer, value)
        }
    }
}

pub struct ClassDescriptor<'a, T: Class> {
    name: &'a str,
    allocate: AllocateCallback<T>,
    call: Option<ConstructorCallCallback>,
    construct: Option<ConstructCallback<T>>,
    methods: Vec<(&'a str, MethodCallback<T>)>
}

impl<'a, T: Class> ClassDescriptor<'a, T> {
    pub fn new<'b, U: Class>(name: &'b str, allocate: AllocateCallback<U>) -> ClassDescriptor<'b, U> {
        ClassDescriptor {
            name: name,
            allocate: allocate,
            call: None,
            construct: None,
            methods: Vec::new()
        }
    }

    pub fn call(mut self, callback: ConstructorCallCallback) -> Self {
        self.call = Some(callback);
        self
    }

    pub fn construct(mut self, callback: ConstructCallback<T>) -> Self {
        self.construct = Some(callback);
        self
    }

    pub fn method(mut self, name: &'a str, callback: MethodCallback<T>) -> Self {
        self.methods.push((name, callback));
        self
    }
}

extern "C" fn drop_internals<T>(internals: *mut c_void) {
    let p: Box<T> = unsafe { Box::from_raw(mem::transmute(internals)) };
    mem::drop(p);
}

pub trait Class: Managed + Any {
    type Internals;

    fn setup<'a, V: Vm<'a>>(_: &mut V) -> VmResult<ClassDescriptor<'a, Self>>;

    fn class<'a, V: Vm<'a>>(vm: &mut V) -> JsResult<'a, JsClass<Self>> {
        let metadata = Self::metadata(vm)?;
        Ok(unsafe { metadata.class(vm) })
    }

    fn describe<'a>(name: &'a str, allocate: AllocateCallback<Self>) -> ClassDescriptor<'a, Self> {
        ClassDescriptor::<Self>::new(name, allocate)
    }
}

unsafe impl<T: Class> This for T {
    fn as_this(h: raw::Local) -> Self {
        Self::from_raw(h)
    }
}

impl<T: Class> Object for T { }

pub(crate) trait ClassInternal: Class {
    fn metadata_opt<'a, V: Vm<'a>>(vm: &mut V) -> Option<ClassMetadata> {
        vm.isolate()
          .class_map()
          .get(&TypeId::of::<Self>())
          .map(|m| m.clone())
    }

    fn metadata<'a, V: Vm<'a>>(vm: &mut V) -> VmResult<ClassMetadata> {
        match Self::metadata_opt(vm) {
            Some(metadata) => Ok(metadata),
            None => Self::create(vm)
        }
    }

    fn create<'a, V: Vm<'a>>(vm: &mut V) -> VmResult<ClassMetadata> {
        let descriptor = Self::setup(vm)?;
        unsafe {
            let isolate: *mut c_void = mem::transmute(vm.isolate());

            let allocate = descriptor.allocate.into_c_callback();
            let construct = descriptor.construct.map(|callback| callback.into_c_callback()).unwrap_or_default();
            let call = descriptor.call.unwrap_or_else(ConstructorCallCallback::default::<Self>).into_c_callback();

            let metadata_pointer = neon_runtime::class::create_base(isolate,
                                                                    allocate,
                                                                    construct,
                                                                    call,
                                                                    drop_internals::<Self::Internals>);

            if metadata_pointer.is_null() {
                return Err(Throw);
            }

            // NOTE: None of the error cases below need to delete the ClassMetadata object, since the
            //       v8::FunctionTemplate has a finalizer that will delete it.

            let class_name = descriptor.name;
            if !neon_runtime::class::set_name(isolate, metadata_pointer, class_name.as_ptr(), class_name.len() as u32) {
                return Err(Throw);
            }

            for (name, method) in descriptor.methods {
                let method: Handle<JsValue> = build(|out| {
                    let callback = method.into_c_callback();
                    neon_runtime::fun::new_template(out, isolate, callback)
                })?;
                if !neon_runtime::class::add_method(isolate, metadata_pointer, name.as_ptr(), name.len() as u32, method.to_raw()) {
                    return Err(Throw);
                }
            }

            let metadata = ClassMetadata {
                pointer: metadata_pointer
            };

            vm.isolate().class_map().set(TypeId::of::<Self>(), metadata);

            Ok(metadata)
        }
    }
}

impl<T: Class> ClassInternal for T { }

impl<T: Class> ValueInternal for T {
    fn is_typeof<Other: Value>(value: Other) -> bool {
        let mut isolate: Isolate = unsafe {
            mem::transmute(neon_runtime::call::current_isolate())
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

impl<T: Class> JsClass<T> {
    pub fn check<U: Value>(&self, v: Handle<U>, msg: &str) -> JsResult<T> {
        let local = v.to_raw();
        if unsafe { neon_runtime::class::check(self.to_raw(), local) } {
            Ok(Handle::new_internal(T::from_raw(local)))
        } else {
            JsError::throw(Kind::TypeError, msg)
        }
    }

    pub fn constructor<'a, V: Vm<'a>>(&self, _: &mut V) -> JsResult<'a, JsFunction<T>> {
        build(|out| {
            unsafe {
                neon_runtime::class::constructor(out, self.to_raw())
            }
        })
    }
}

impl<'a, T: Class> Borrow for &'a T {
    type Target = &'a mut T::Internals;

    fn try_borrow<'b>(self, guard: &'b VmGuard<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        unsafe {
            let ptr: *mut c_void = neon_runtime::class::get_instance_internals(self.to_raw());
            Ref::new(guard, mem::transmute(ptr))
        }
    }
}

impl<'a, T: Class> Borrow for &'a mut T {
    type Target = &'a mut T::Internals;

    fn try_borrow<'b>(self, guard: &'b VmGuard<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        (self as &'a T).try_borrow(guard)
    }
}

impl<'a, T: Class> BorrowMut for &'a mut T {
    fn try_borrow_mut<'b>(self, guard: &'b VmGuard<'b>) -> Result<RefMut<'b, Self::Target>, LoanError> {
        unsafe {
            let ptr: *mut c_void = neon_runtime::class::get_instance_internals(self.to_raw());
            RefMut::new(guard, mem::transmute(ptr))
        }
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
