//! Types and traits representing JavaScript classes backed by Rust data.

pub(crate) mod internal;

use std::any::{Any, TypeId};
use std::mem;
use std::os::raw::c_void;
use std::slice;
use std::collections::HashMap;
use neon_runtime;
use neon_runtime::raw;
use neon_runtime::call::CCallback;
use context::{Context, Lock, CallbackInfo};
use context::internal::Env;
use result::{NeonResult, JsResult, Throw};
use borrow::{Borrow, BorrowMut, Ref, RefMut, LoanError};
use handle::{Handle, Managed};
use types::{Value, JsFunction, JsValue, build};
use types::internal::ValueInternal;
use object::{Object, This};
use self::internal::{ClassMetadata, MethodCallback, ConstructorCallCallback, AllocateCallback, ConstructCallback};

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
    #[cfg(feature = "legacy-runtime")]
    fn as_this(h: raw::Local) -> Self {
        Self::from_raw(Env::current(), h)
    }

    #[cfg(feature = "napi-1")]
    fn as_this(env: Env, h: raw::Local) -> Self {
        Self::from_raw(env, h)
    }
}

impl<T: Class> Object for T { }

pub(crate) trait ClassInternal: Class {
    fn metadata_opt<'a, C: Context<'a>>(cx: &mut C) -> Option<ClassMetadata> {
        cx.env()
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
            let env = cx.env().to_raw();

            let allocate = descriptor.allocate.into_c_callback();
            let construct = descriptor.construct.map(|callback| callback.into_c_callback()).unwrap_or_default();
            let call = descriptor.call.unwrap_or_else(ConstructorCallCallback::default::<Self>).into_c_callback();

            let metadata_pointer = neon_runtime::class::create_base(env,
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
            if !neon_runtime::class::set_name(env, metadata_pointer, class_name.as_ptr(), class_name.len() as u32) {
                return Err(Throw);
            }

            for (name, method) in descriptor.methods {
                let method: Handle<JsValue> = build(cx.env(), |out| {
                    let callback = method.into_c_callback();
                    neon_runtime::fun::new_template(out, env, callback)
                })?;
                if !neon_runtime::class::add_method(env, metadata_pointer, name.as_ptr(), name.len() as u32, method.to_raw()) {
                    return Err(Throw);
                }
            }

            let metadata = ClassMetadata {
                pointer: metadata_pointer
            };

            cx.env().class_map().set(TypeId::of::<Self>(), metadata);

            Ok(metadata)
        }
    }
}

impl<T: Class> ClassInternal for T { }

impl<T: Class> ValueInternal for T {
    fn name() -> String {
        let mut isolate: Env = unsafe {
            mem::transmute(neon_runtime::call::current_isolate())
        };
        let raw_isolate = unsafe { mem::transmute(isolate) };
        let map = isolate.class_map();
        match map.get(&TypeId::of::<T>()) {
            None => "unknown".to_string(),
            Some(ref metadata) => {
                let mut chars = std::ptr::null_mut();

                let buf = unsafe {
                    let len = neon_runtime::class::get_name(&mut chars, raw_isolate, metadata.pointer);

                    slice::from_raw_parts_mut(chars, len)
                };

                String::from_utf8_lossy(buf).to_string()
            }
        }
    }

    fn is_typeof<Other: Value>(mut env: Env, value: Other) -> bool {
        let map = env.class_map();
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

/// A dynamically computed callback that can be passed through C to the engine.
/// This type makes it possible to export a dynamically computed Rust function
/// as a pair of 1) a raw pointer to the dynamically computed function, and 2)
/// a static function that knows how to transmute that raw pointer and call it.
pub(crate) trait Callback<T: Clone + Copy + Sized>: Sized {
    /// Extracts the computed Rust function and invokes it. The Neon runtime
    /// ensures that the computed function is provided as the extra data field,
    /// wrapped as a V8 External, in the `CallbackInfo` argument.
    extern "C" fn invoke(env: Env, info: CallbackInfo<'_>) -> T;

    /// See `invoke`. This is used by the non-n-api implementation, so that every impl for this
    /// trait doesn't need to provide two versions of `invoke`.
    #[cfg(feature = "legacy-runtime")]
    #[doc(hidden)]
    extern "C" fn invoke_compat(info: CallbackInfo<'_>) -> T {
        Self::invoke(Env::current(), info)
    }

    /// Converts the callback to a raw void pointer.
    fn as_ptr(self) -> *mut c_void;

    /// Exports the callback as a pair consisting of the static `Self::invoke`
    /// method and the computed callback, both converted to raw void pointers.
    fn into_c_callback(self) -> CCallback {
        #[cfg(feature = "napi-1")]
        let invoke = Self::invoke;
        #[cfg(feature = "legacy-runtime")]
        let invoke = Self::invoke_compat;
        CCallback {
            static_callback: unsafe { mem::transmute(invoke as usize) },
            dynamic_callback: self.as_ptr()
        }
    }
}
