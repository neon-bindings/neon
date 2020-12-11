use std::sync::{Arc, Mutex};

use neon_runtime::raw;

use crate::context::{Context, TaskContext};
use crate::context::internal::Env;
use crate::handle::{Managed, Handle};
use crate::types::internal::ValueInternal;
use crate::types::{Object, Value};

use super::JsValue;

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

impl Object for JsPromise { }

// FIXME: This should be a state machine enum
struct JsPromiseFutureInner<F, R> {
    callback: Option<F>,
    result: Option<R>,
    waker: Option<std::task::Waker>,
}

pub struct JsPromiseFuture<F, R> {
    inner: Arc<Mutex<JsPromiseFutureInner<F, R>>>,
}

impl<F, R> std::future::Future for JsPromiseFuture<F, R>
where
    R: Send + 'static,
    F: for<'c> FnOnce(
        TaskContext<'c>,
        Result<Handle<'c, JsValue>, Handle<'c, JsValue>>,
    ) -> R + Send + 'static,
{
    type Output = R;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut inner = self.inner.lock().unwrap();

        if let Some(result) = inner.result.take() {
            std::task::Poll::Ready(result)
        } else {
            inner.waker = Some(cx.waker().clone());
            std::task::Poll::Pending
        }
    }
}

impl<F, R> JsPromiseFuture<F, R>
where
    R: Send + 'static,
    F: for<'c> FnOnce(
        TaskContext<'c>,
        Result<Handle<'c, JsValue>, Handle<'c, JsValue>>,
    ) -> R + Send + 'static,
{
    pub fn new<'a, C, T>(
        cx: &mut C,
        maybe_promise: Handle<T>,
        callback: F,
    ) -> Self
    where
        C: Context<'a>,
        T: Value,
    {
        // Support promise-like objects by creating a new promise and
        // immediately resolving.
        let (promise, deferred) = cx.promise();

        deferred.resolve(cx, maybe_promise);

        let inner = Arc::new(Mutex::new(JsPromiseFutureInner {
            callback: Some(callback),
            result: None,
            waker: None,
        }));

        let make_callback = |success| {
            let inner = inner.clone();

            move |env, value| {
                let env = unsafe { std::mem::transmute(env) };

                TaskContext::with_context(env, move |cx| {
                    let mut inner = inner.lock().unwrap(); 
                    let callback = inner.callback.take().unwrap();
                    let value = JsValue::new_internal(value);
                    let value = if success {
                        Ok(value)
                    } else {
                        Err(value)
                    };

                    inner.result = Some(callback(cx, value));

                    if let Some(waker) = inner.waker.take() {
                        waker.wake();
                    }
                });
            }
        };

        unsafe {
            neon_runtime::promise::adapter(
                cx.env().to_raw(),
                promise.to_raw(),
                make_callback(true),
                make_callback(false),
            )
        }

        JsPromiseFuture { inner }
    }
}
