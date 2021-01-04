//! Helper to run a callback in the libuv main thread.

use std::os::raw::c_void;

use types::*;
use handle::{Handle, Managed};
use neon_runtime;
use neon_runtime::raw;
use std::sync::Arc;
use context::Context;
use context::internal::ContextInternal;

type EventContext<'a> = crate::context::TaskContext<'a>;

struct EventHandlerInner(*mut c_void);

unsafe impl Send for EventHandlerInner {}
unsafe impl Sync for EventHandlerInner {}

impl Drop for EventHandlerInner {
    fn drop(&mut self) {
        unsafe {
            neon_runtime::handler::delete(self.0);
        }
    }
}

#[derive(Clone)]
pub struct EventHandler(Arc<EventHandlerInner>);

impl EventHandler {
    #[cfg(feature = "napi-1")]
    pub fn new<'a, C: Context<'a>, T: Value>(cx: &C, this: Handle<T>, callback: Handle<JsFunction>) -> Self {
        unimplemented!()
    }

    #[cfg(feature = "legacy-runtime")]
    pub fn new<'a, C: Context<'a>, T: Value>(cx: &C, this: Handle<T>, callback: Handle<JsFunction>) -> Self {
        let cb = unsafe {
            neon_runtime::handler::new(cx.env().to_raw(), this.to_raw(), callback.to_raw())
        };
        EventHandler(Arc::new(EventHandlerInner(cb)))
    }

    pub fn schedule<T, F>(&self, arg_cb: F)
        where T: Value,
              F: for<'a> FnOnce(&mut EventContext<'a>) -> Vec<Handle<'a, T>>,
              F: Send + 'static {
        self.schedule_with(move |cx, this, callback| {
            let args = arg_cb(cx);
            let _result = callback.call(cx, this, args);
        })
    }

    fn schedule_internal<F>(&self, cb: F)
        where F: FnOnce(&mut EventContext, Handle<JsValue>, Handle<JsFunction>),
              F: Send + 'static {
        let callback = Box::into_raw(Box::new(cb)) as *mut c_void;
        unsafe {
            neon_runtime::handler::schedule((*self.0).0, callback, handle_callback::<F>);
        }
    }

    pub fn schedule_with<F>(&self, arg_cb: F)
        where F: FnOnce(&mut EventContext, Handle<JsValue>, Handle<JsFunction>),
              F: Send + 'static {
        // HACK: Work around for race condition in `close`. `EventHandler` cannot be
        // dropped until all callbacks have executed.
        // NOTE: This will still leak memory if the callback is never called
        let cloned_cb = self.clone();

        self.schedule_internal(move |cx, this, cb| {
            arg_cb(cx, this, cb);
            let _ = cloned_cb;
        });
    }
}

unsafe extern "C" fn handle_callback<F>(this: raw::Local, func: raw::Local, callback: *mut c_void)
    where F: FnOnce(&mut EventContext, Handle<JsValue>, Handle<JsFunction>), F: Send + 'static {
    EventContext::with(|mut cx: EventContext| {
        let this = JsValue::new_internal(this);
        let func: Handle<JsFunction> = Handle::new_internal(JsFunction::from_raw(cx.env(), func));
        let callback: Box<F> = Box::from_raw(callback as *mut _);
        callback(&mut cx, this, func);
    })
}
