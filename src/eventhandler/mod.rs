//! Helper to run a callback in the libuv main thread.

use std::mem;
use std::os::raw::c_void;

use types::*;
use handle::{Handle, Managed};
use context::*;
use neon_runtime;
use neon_runtime::raw;
use std::sync::Arc;

struct EventHandlerInner(*mut c_void);

unsafe impl Send for EventHandlerInner {}
unsafe impl Sync for EventHandlerInner {}

impl Drop for EventHandlerInner {
    fn drop(&mut self) {
        unsafe {
            neon_runtime::threadsafecb::delete(self.0);
        }
    }
}

#[derive(Clone)]
pub struct EventHandler(Arc<EventHandlerInner>);

impl EventHandler {
    pub fn new(callback: Handle<JsFunction>) -> Self {
        TaskContext::with(|mut cx: TaskContext| {
            let cb = unsafe {
                neon_runtime::threadsafecb::new(cx.global().to_raw(), callback.to_raw())
            };
            EventHandler(Arc::new(EventHandlerInner(cb)))
        })
    }

    pub fn bind<T: Value>(this: Handle<T>, callback: Handle<JsFunction>) -> Self {
        let cb = unsafe {
            neon_runtime::threadsafecb::new(this.to_raw(), callback.to_raw())
        };
        EventHandler(Arc::new(EventHandlerInner(cb)))
    }

    pub fn schedule<T, F>(&self, arg_cb: F)
        where T: Value,
              F: for<'a> FnOnce(&mut TaskContext<'a>) -> Vec<Handle<'a, T>>,
              F: Send + 'static {
        let callback = Box::into_raw(Box::new(arg_cb)) as *mut c_void;
        unsafe {
            neon_runtime::threadsafecb::call((*self.0).0, callback, handle_callback::<T, F>);
        }
    }

    pub fn schedule_with<F>(&self, arg_cb: F)
        where F: FnOnce(&mut TaskContext, Handle<JsValue>, Handle<JsFunction>),
              F: Send + 'static {
        let callback = Box::into_raw(Box::new(arg_cb)) as *mut c_void;
        unsafe {
            neon_runtime::threadsafecb::call((*self.0).0, callback, handle_callback_with::<F>);
        }
    }
}

unsafe extern "C" fn handle_callback<T, F>(this: raw::Local, func: raw::Local, callback: *mut c_void)
    where T: Value, F: for<'a> FnOnce(&mut TaskContext<'a>) -> Vec<Handle<'a, T>>, F: Send + 'static {
    TaskContext::with(|mut cx: TaskContext| {
        let this = JsValue::new_internal(this);
        let func: Handle<JsFunction> = Handle::new_internal(JsFunction::from_raw(func));
        let callback: Box<F> = Box::from_raw(mem::transmute(callback));
        let args = callback(&mut cx);
        let _result = func.call(&mut cx, this, args);
    })
}

unsafe extern "C" fn handle_callback_with<F>(this: raw::Local, func: raw::Local, callback: *mut c_void)
    where F: FnOnce(&mut TaskContext, Handle<JsValue>, Handle<JsFunction>), F: Send + 'static {
    TaskContext::with(|mut cx: TaskContext| {
        let this = JsValue::new_internal(this);
        let func: Handle<JsFunction> = Handle::new_internal(JsFunction::from_raw(func));
        let callback: Box<F> = Box::from_raw(mem::transmute(callback));
        callback(&mut cx, this, func);
    })
}
