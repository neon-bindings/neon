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
