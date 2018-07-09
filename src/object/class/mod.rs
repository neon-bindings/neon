//! Types and traits representing JavaScript classes backed by Rust data.

use std::any::{Any, TypeId};
use std::mem;
use std::os::raw::c_void;
use std::slice;
use std::collections::HashMap;
use neon_runtime;
use neon_runtime::raw;
use neon_runtime::call::CCallback;
use cx::{Context, Lock, CallbackInfo};
use cx::internal::Isolate;
use result::{NeonResult, Throw};
use borrow::{Borrow, BorrowMut, Ref, RefMut, LoanError};
use value::{JsResult, Value, JsFunction, JsValue, Handle, Managed, build};
use value::internal::ValueInternal;
use object::{Object, This};
use self::internal::{ClassMetadata, MethodCallback, ConstructorCallCallback, AllocateCallback, ConstructCallback};

pub(crate) mod internal {
    use std::mem;
    use std::os::raw::c_void;
    use std::ptr::null_mut;
    use neon_runtime;
    use neon_runtime::raw;
    use super::{Class, ClassInternal, Callback};
    use value::{JsResult, JsValue, JsObject, JsFunction, JsUndefined, Handle, Managed, build};
    use cx::{CallbackInfo, CallContext, Context};
    use cx::internal::ContextInternal;
    use result::{NeonResult, Throw};
    use value::error::convert_panics;

    #[repr(C)]
    pub struct MethodCallback<T: Class>(pub fn(CallContext<T>) -> JsResult<JsValue>);

    impl<T: Class> Callback<()> for MethodCallback<T> {
        extern "C" fn invoke(info: &CallbackInfo) {
            unsafe {
                info.with_cx::<T, _, _>(|mut cx| {
                    let data = info.data();
                    let this: Handle<JsValue> = Handle::new_internal(JsValue::from_raw(info.this(&mut cx)));
                    if !this.is_a::<T>() {
                        if let Ok(metadata) = T::metadata(&mut cx) {
                            neon_runtime::class::throw_this_error(mem::transmute(cx.isolate()), metadata.pointer);
                        }
                        return;
                    };
                    let dynamic_callback: fn(CallContext<T>) -> JsResult<JsValue> =
                        mem::transmute(neon_runtime::fun::get_dynamic_callback(data.to_raw()));
                    if let Ok(value) = convert_panics(|| { dynamic_callback(cx) }) {
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
            fn callback<T: Class>(mut cx: CallContext<JsValue>) -> JsResult<JsValue> {
                unsafe {
                    if let Ok(metadata) = T::metadata(&mut cx) {
                        neon_runtime::class::throw_call_error(mem::transmute(cx.isolate()), metadata.pointer);
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
                info.with_cx(|cx| {
                    let data = info.data();
                    let kernel: fn(CallContext<JsValue>) -> JsResult<JsValue> =
                        mem::transmute(neon_runtime::class::get_call_kernel(data.to_raw()));
                    if let Ok(value) = convert_panics(|| { kernel(cx) }) {
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
    pub struct AllocateCallback<T: Class>(pub fn(CallContext<JsUndefined>) -> NeonResult<T::Internals>);

    impl<T: Class> Callback<*mut c_void> for AllocateCallback<T> {
        extern "C" fn invoke(info: &CallbackInfo) -> *mut c_void {
            unsafe {
                info.with_cx(|cx| {
                    let data = info.data();
                    let kernel: fn(CallContext<JsUndefined>) -> NeonResult<T::Internals> =
                        mem::transmute(neon_runtime::class::get_allocate_kernel(data.to_raw()));
                    if let Ok(value) = convert_panics(|| { kernel(cx) }) {
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
    pub struct ConstructCallback<T: Class>(pub fn(CallContext<T>) -> NeonResult<Option<Handle<JsObject>>>);

    impl<T: Class> Callback<bool> for ConstructCallback<T> {
        extern "C" fn invoke(info: &CallbackInfo) -> bool {
            unsafe {
                info.with_cx(|cx| {
                    let data = info.data();
                    let kernel: fn(CallContext<T>) -> NeonResult<Option<Handle<JsObject>>> =
                        mem::transmute(neon_runtime::class::get_construct_kernel(data.to_raw()));
                    match convert_panics(|| { kernel(cx) }) {
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
        pub unsafe fn constructor<'a, T: Class, C: Context<'a>>(&self, cx: &mut C) -> JsResult<'a, JsFunction<T>> {
            build(|out| {
                neon_runtime::class::metadata_to_constructor(out, mem::transmute(cx.isolate()), self.pointer)
            })
        }

        pub unsafe fn has_instance(&self, value: raw::Local) -> bool {
            neon_runtime::class::has_instance(self.pointer, value)
        }
    }
}

pub(crate) struct ClassMap {
    map: HashMap<TypeId, ClassMetadata>
}

impl ClassMap {
    pub(crate) fn new() -> ClassMap {
        ClassMap {
            map: HashMap::new()
        }
    }

    pub(crate) fn get(&self, key: &TypeId) -> Option<&ClassMetadata> {
        self.map.get(key)
    }

    pub(crate) fn set(&mut self, key: TypeId, val: ClassMetadata) {
        self.map.insert(key, val);
    }
}

#[doc(hidden)]
pub struct ClassDescriptor<'a, T: Class> {
    name: &'a str,
    allocate: AllocateCallback<T>,
    call: Option<ConstructorCallCallback>,
    construct: Option<ConstructCallback<T>>,
    methods: Vec<(&'a str, MethodCallback<T>)>
}

impl<'a, T: Class> ClassDescriptor<'a, T> {

    /// Constructs a new minimal `ClassDescriptor` with a name and allocator.
    pub fn new<'b, U: Class>(name: &'b str, allocate: AllocateCallback<U>) -> ClassDescriptor<'b, U> {
        ClassDescriptor {
            name: name,
            allocate: allocate,
            call: None,
            construct: None,
            methods: Vec::new()
        }
    }

    /// Adds `[[Call]]` behavior for the constructor to this class descriptor.
    pub fn call(mut self, callback: ConstructorCallCallback) -> Self {
        self.call = Some(callback);
        self
    }

    /// Adds `[[Construct]]` behavior for the constructor to this class descriptor.
    pub fn construct(mut self, callback: ConstructCallback<T>) -> Self {
        self.construct = Some(callback);
        self
    }

    /// Adds a method to this class descriptor.
    pub fn method(mut self, name: &'a str, callback: MethodCallback<T>) -> Self {
        self.methods.push((name, callback));
        self
    }

}

extern "C" fn drop_internals<T>(internals: *mut c_void) {
    let p: Box<T> = unsafe { Box::from_raw(mem::transmute(internals)) };
    mem::drop(p);
}

/// The trait implemented by Neon classes.
/// 
/// This trait is not intended to be implemented manually; it is implemented automatically by
/// creating a class with the `class` syntax of the `declare_types!` macro.
pub trait Class: Managed + Any {
    type Internals;

    #[doc(hidden)]
    fn setup<'a, C: Context<'a>>(_: &mut C) -> NeonResult<ClassDescriptor<'a, Self>>;

    /// Produces a handle to the constructor function for this class.
    fn constructor<'a, C: Context<'a>>(cx: &mut C) -> JsResult<'a, JsFunction<Self>> {
        let metadata = Self::metadata(cx)?;
        unsafe { metadata.constructor(cx) }
    }

    /// Convenience method for constructing new instances of this class without having to extract the constructor function.
    fn new<'a, 'b, C: Context<'a>, A, AS>(cx: &mut C, args: AS) -> JsResult<'a, Self>
        where A: Value + 'b,
              AS: IntoIterator<Item=Handle<'b, A>>
    {
        let constructor = Self::constructor(cx)?;
        constructor.construct(cx, args)
    }

    #[doc(hidden)]
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
    fn metadata_opt<'a, C: Context<'a>>(cx: &mut C) -> Option<ClassMetadata> {
        cx.isolate()
          .class_map()
          .get(&TypeId::of::<Self>())
          .map(|m| m.clone())
    }

    fn metadata<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<ClassMetadata> {
        match Self::metadata_opt(cx) {
            Some(metadata) => Ok(metadata),
            None => Self::create(cx)
        }
    }

    fn create<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<ClassMetadata> {
        let descriptor = Self::setup(cx)?;
        unsafe {
            let isolate: *mut c_void = mem::transmute(cx.isolate());

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

            cx.isolate().class_map().set(TypeId::of::<Self>(), metadata);

            Ok(metadata)
        }
    }
}

impl<T: Class> ClassInternal for T { }

impl<T: Class> ValueInternal for T {
    fn name() -> String {
        let mut isolate: Isolate = unsafe {
            mem::transmute(neon_runtime::call::current_isolate())
        };
        let raw_isolate = unsafe { mem::transmute(isolate) };
        let map = isolate.class_map();
        match map.get(&TypeId::of::<T>()) {
            None => "unknown".to_string(),
            Some(ref metadata) => unsafe {
                let mut chars: *mut u8 = mem::uninitialized();
                let mut len: usize = mem::uninitialized();
                neon_runtime::class::get_name(&mut chars, &mut len, raw_isolate, metadata.pointer);
                String::from_utf8_lossy(slice::from_raw_parts_mut(chars, len)).to_string()
            }
        }
    }

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

impl<'a, T: Class> Borrow for &'a T {
    type Target = &'a mut T::Internals;

    fn try_borrow<'b>(self, lock: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        unsafe {
            let ptr: *mut c_void = neon_runtime::class::get_instance_internals(self.to_raw());
            Ref::new(lock, mem::transmute(ptr))
        }
    }
}

impl<'a, T: Class> Borrow for &'a mut T {
    type Target = &'a mut T::Internals;

    fn try_borrow<'b>(self, lock: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        (self as &'a T).try_borrow(lock)
    }
}

impl<'a, T: Class> BorrowMut for &'a mut T {
    fn try_borrow_mut<'b>(self, lock: &'b Lock<'b>) -> Result<RefMut<'b, Self::Target>, LoanError> {
        unsafe {
            let ptr: *mut c_void = neon_runtime::class::get_instance_internals(self.to_raw());
            RefMut::new(lock, mem::transmute(ptr))
        }
    }
}

/// A dynamically computed callback that can be passed through C to the JS VM.
/// This type makes it possible to export a dynamically computed Rust function
/// as a pair of 1) a raw pointer to the dynamically computed function, and 2)
/// a static function that knows how to transmute that raw pointer and call it.
pub(crate) trait Callback<T: Clone + Copy + Sized>: Sized {

    /// Extracts the computed Rust function and invokes it. The Neon runtime
    /// ensures that the computed function is provided as the extra data field,
    /// wrapped as a V8 External, in the `CallbackInfo` argument.
    extern "C" fn invoke(info: &CallbackInfo) -> T;

    /// Converts the callback to a raw void pointer.
    fn as_ptr(self) -> *mut c_void;

    /// Exports the callback as a pair consisting of the static `Self::invoke`
    /// method and the computed callback, both converted to raw void pointers.
    fn into_c_callback(self) -> CCallback {
        CCallback {
            static_callback: unsafe { mem::transmute(Self::invoke as usize) },
            dynamic_callback: self.as_ptr()
        }
    }
}
