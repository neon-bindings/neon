use std::mem;
use std::os::raw::c_void;
use std::ptr::null_mut;
use neon_runtime;
use neon_runtime::raw;
use super::{Class, ClassInternal, Callback};
use context::{CallbackInfo, CallContext, Context};
use context::internal::ContextInternal;
use result::{NeonResult, Throw};
use types::{JsFunction, JsValue, JsObject, JsUndefined, Managed, Value};
use types::error::convert_panics;

#[repr(C)]
pub struct MethodCallback<T: Class>(pub fn(CallContext<T>) -> NeonResult<&JsValue>);

impl<T: Class> Callback<()> for MethodCallback<T> {
    extern "C" fn invoke(info: &CallbackInfo) {
        unsafe {
            info.with_cx::<T, _, _>(|mut cx| {
                let data = info.data(&mut cx);
                let this = info.this(&mut cx);
                if !this.is_a::<T>() {
                    let isolate = { cx.isolate().to_raw() };
                    if let Ok(metadata) = T::metadata(&mut cx) {
                        neon_runtime::class::throw_this_error(isolate, metadata.pointer);
                    }
                    return;
                }
                let dynamic_callback: fn(CallContext<T>) -> NeonResult<&JsValue> =
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
pub struct ConstructorCallCallback(pub fn(CallContext<JsValue>) -> NeonResult<&JsValue>);

impl ConstructorCallCallback {
    pub(crate) fn default<T: Class>() -> Self {
        fn callback<T: Class>(mut cx: CallContext<JsValue>) ->  NeonResult<&JsValue> {
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
            info.with_cx(|mut cx| {
                let data = info.data(&mut cx);
                let kernel: fn(CallContext<JsValue>) -> NeonResult<&JsValue> =
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
            info.with_cx(|mut cx| {
                let data = info.data(&mut cx);
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
pub struct ConstructCallback<T: Class>(pub fn(CallContext<T>) -> NeonResult<Option<&JsObject>>);

impl<T: Class> Callback<bool> for ConstructCallback<T> {
    extern "C" fn invoke(info: &CallbackInfo) -> bool {
        unsafe {
            info.with_cx(|mut cx| {
                let data = info.data(&mut cx);
                let kernel: fn(CallContext<T>) -> NeonResult<Option<&JsObject>> =
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
    pub unsafe fn constructor<'a, T: Class, C: Context<'a>>(&self, cx: &mut C) -> NeonResult<&'a JsFunction<T>> {
        let isolate = { cx.isolate().to_raw() };
        cx.new(|out| {
            neon_runtime::class::metadata_to_constructor(out, isolate, self.pointer)
        })
    }

    pub unsafe fn has_instance(&self, value: &raw::Persistent) -> bool {
        neon_runtime::class::has_instance(self.pointer, value)
    }
}
