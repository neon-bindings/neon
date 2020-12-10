use neon_runtime::raw;

use crate::context::Context;
use crate::context::internal::Env;
use crate::handle::{Managed, Handle};
use crate::types::internal::ValueInternal;
use crate::types::Value;

/// A JavaScript Promise object
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsPromise(raw::Local);

#[repr(C)]
pub struct Deferred(*mut std::ffi::c_void);

unsafe impl Send for Deferred {}

impl Deferred {
    pub fn resolve<'a, C: Context<'a>, T: Value>(
        self,
        cx: &mut C, value: Handle<T>,
    ) {
        unsafe {
            neon_runtime::promise::resolve(
                cx.env().to_raw(),
                self.0 as *mut _,
                value.to_raw(),
            )
        }
    }

    pub fn reject<'a, C: Context<'a>, T: Value>(
        self,
        cx: &mut C, value: Handle<T>,
    ) {
        unsafe {
            neon_runtime::promise::reject(
                cx.env().to_raw(),
                self.0 as *mut _,
                value.to_raw(),
            )
        }
    }
}

impl JsPromise {
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> (Handle<'a, JsPromise>, Deferred) {
        let (deferred, local) = unsafe {
            neon_runtime::promise::new(cx.env().to_raw())
        };

        let promise = Handle::new_internal(JsPromise(local));
        let deferred = Deferred(deferred as *mut _);

        (promise, deferred)
    }
}

impl Value for JsPromise { }

impl Managed for JsPromise {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(_: Env, h: raw::Local) -> Self { JsPromise(h) }
}

impl ValueInternal for JsPromise {
    fn name() -> String { "Promise".to_string() }

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool {
        unsafe { neon_runtime::tag::is_promise(env.to_raw(), other.to_raw()) }
    }
}
