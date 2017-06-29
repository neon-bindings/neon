use std::marker::{Send, Sized};
use std::mem;
use std::os::raw::c_void;

use js::{Value, JsFunction};
use mem::Handle;
use internal::mem::Managed;
use internal::scope::{Scope, RootScope, RootScopeInternal};
use internal::vm::{JsResult, Isolate, IsolateInternal};
use neon_runtime;
use neon_runtime::raw;

pub trait Task: Send + Sized {
    type Output: Send;
    type Error: Send;
    type JsEvent: Value;

    fn perform(&self) -> Result<Self::Output, Self::Error>;

    fn complete<'a, T: Scope<'a>>(self, scope: &'a mut T, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent>;

    fn schedule(self, callback: Handle<JsFunction>) {
        let boxed_self = Box::new(self);
        let self_raw = Box::into_raw(boxed_self);
        let callback_raw = callback.to_raw();
        unsafe {
            neon_runtime::task::schedule(mem::transmute(self_raw),
                                         perform_task::<Self>,
                                         complete_task::<Self>,
                                         callback_raw);
        }
    }
}

unsafe extern "C" fn perform_task<T: Task>(task: *mut c_void) -> *mut c_void {
    let task: Box<T> = Box::from_raw(mem::transmute(task));
    let result = task.perform();
    Box::into_raw(task);
    mem::transmute(Box::into_raw(Box::new(result)))
}

unsafe extern "C" fn complete_task<T: Task>(task: *mut c_void, result: *mut c_void, out: &mut raw::Local) {
    let result: Result<T::Output, T::Error> = *Box::from_raw(mem::transmute(result));
    let task: Box<T> = Box::from_raw(mem::transmute(task));

    // The neon::Task::complete() method installs an outer v8::HandleScope
    // that is responsible for managing the out pointer, so it's safe to
    // create the RootScope here without creating a local v8::HandleScope.
    let mut scope = RootScope::new(Isolate::current());
    if let Ok(result) = task.complete(&mut scope, result) {
        *out = result.to_raw();
    }
}
