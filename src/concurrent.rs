//! Traits for defining Rust _tasks_ to be executed in either a Rust background thread or on Node's libuv thread pool.

extern crate crossbeam;
extern crate neon_runtime;

use std::error::Error;
use std::marker::{Send, Sized};
use std::mem;
use std::os::raw::c_void;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use js::error::{JsError, Kind};
use js::{JsFunction, JsUndefined, Value};
use mem::{Handle, Managed};
use scope::{RootScope, Scope};
use vm::internal::Isolate;
use vm::{Call, JsResult};

use neon_runtime::async_callback::AsyncCallback;
use neon_runtime::raw;

use async::AsyncHandle;

use concurrent::crossbeam::sync::SegQueue;

#[derive(Debug)]
pub enum Message<T, E, C> {
    Event(T),
    Error(E),
    Complete(C),
}

#[derive(Debug)]
pub struct AsyncContext<T> {
    callback: AsyncCallback,
    async_handle: AsyncHandle,
    queue: SegQueue<T>,
    completion_sender: Sender<()>,
    completion_receiver: Receiver<()>,
}

impl<T> AsyncContext<T> {
    fn new(callback: Handle<JsFunction>) -> Self {
        let async_handle = AsyncHandle::new();
        let queue: SegQueue<T> = SegQueue::new();
        let (completion_sender, completion_receiver): (Sender<()>, Receiver<()>) = channel();

        AsyncContext {
            callback: AsyncCallback::new(callback.to_raw()),
            async_handle,
            queue,
            completion_sender,
            completion_receiver,
        }
    }
}

pub trait Worker: Send + Sized + 'static {
    type Complete: Send;
    type Error: Send + Error;
    type Event: Send;
    type IncomingEvent: Send;
    type IncomingEventError: Error + Sized;

    type JsComplete: Value;
    type JsEvent: Value;

    fn complete<'a, T: Scope<'a>>(
        &'a self,
        scope: &'a mut T,
        result: Result<&Self::Complete, &Self::Error>,
    ) -> JsResult<Self::JsComplete>;

    fn event<'a, T: Scope<'a>>(
        &'a self,
        scope: &'a mut T,
        value: &Self::Event,
    ) -> JsResult<Self::JsEvent>;

    fn on_incoming_event<'a>(call: Call<'a>) -> Result<Self::IncomingEvent, Self::IncomingEventError>;

    fn perform<N: FnMut(Message<Self::Event, Self::Error, Self::Complete>)>(
        &self,
        emit: N,
        receiver: Receiver<Self::IncomingEvent>,
    );

    fn spawn<'a, T: Scope<'a>>(
        self,
        scope: &'a mut T,
        callback: Handle<JsFunction>,
    ) -> JsResult<'a, JsFunction> {
        let AsyncContext {
            mut callback,
            async_handle,
            queue,
            completion_sender,
            completion_receiver,
        } = AsyncContext::new(callback);

        let (event_sender, event_receiver): (
            Sender<Self::IncomingEvent>,
            Receiver<Self::IncomingEvent>,
        ) = channel();

        let _ = thread::spawn(move || {
            let callback = |value: Message<Self::Event, Self::Error, Self::Complete>| {
                // From http://docs.libuv.org/en/v1.x/async.html
                // "Warning: libuv will coalesce calls to uv_async_send(), that is, not every call
                // to it will yield an execution of the callback. For example: if uv_async_send()
                // is called 5 times in a row before the callback is called, the callback will only
                // be called once. If uv_async_send() is called again after the callback was called,
                // it will be called again."
                //
                // To prevent the coalescing behavior from dropping events, we store all events
                // in a queue and then drain it once the event loop wakes up again.
                queue.push(value);

                // Runs on the main thread!
                async_handle.wake_event_loop(|| {
                    let mut is_complete = false;
                    // Drain the entire queue
                    while !is_complete && !queue.is_empty() {
                        let output = queue.try_pop();
                        if output.is_none() {
                            break;
                        }
                        let mut scope = RootScope::new(Isolate::current());
                        scope.with(|scope| match output.unwrap() {
                            Message::Event(next_value) => {
                                let result = self.event(scope, &next_value);
                                callback.call(vec![
                                    JsUndefined::new().to_raw(),
                                    JsUndefined::new().to_raw(),
                                    result.unwrap().to_raw(),
                                ]);
                            }
                            Message::Error(error_value) => {
                                let result = self.complete(scope, Err(&error_value));
                                callback.call(vec![
                                    result.unwrap().to_raw(),
                                    JsUndefined::new().to_raw(),
                                    JsUndefined::new().to_raw(),
                                ]);
                            }
                            Message::Complete(complete_value) => {
                                is_complete = true;
                                let result = self.complete(scope, Ok(&complete_value));
                                callback.call(vec![
                                    JsUndefined::new().to_raw(),
                                    result.unwrap().to_raw(),
                                    JsUndefined::new().to_raw(),
                                ]);
                                callback.destroy();
                                let _ = completion_sender.send(());
                            }
                        })
                    }
                });
            };

            self.perform(callback, event_receiver);

            // Block until the main thread calls the complete callback.
            // Otherwise, all these references will be garbage!
            completion_receiver.recv().unwrap();
        });

        JsFunction::new(
            scope,
            Box::new(move |inner| {
                Self::on_incoming_event(inner)
                    .or_else(|error| JsError::throw(Kind::Error, error.description()))
                    .and_then(|incoming| Ok(event_sender.send(incoming)))
                    .and_then(|_| Ok(JsUndefined::new()))
                    .or_else(|_| {
                        JsError::throw(
                            Kind::Error,
                            "Cannot communicate with Task thread. Has the Task completed or terminated early?",
                        )
                    })
            }),
        )
    }
}

pub trait Task: Send + Sized + 'static {
    type Complete: Send;
    type Error: Send + Error;

    type JsComplete: Value;

    fn perform(&self) -> Result<Self::Complete, Self::Error>;

    fn complete<'a, S: Scope<'a>>(
        &'a self,
        scope: &'a mut S,
        result: Result<&Self::Complete, &Self::Error>,
    ) -> JsResult<Self::JsComplete>;

    fn run<'a, T: Scope<'a>>(
        self,
        scope: &'a mut T,
        callback: Handle<JsFunction>,
    ) -> JsResult<'a, JsFunction> {
        <Self as Worker>::spawn(self, scope, callback)
    }

    fn run_uv(self, callback: Handle<JsFunction>) {
        let boxed_self = Box::new(self);
        let self_raw = Box::into_raw(boxed_self);
        let callback_raw = callback.to_raw();
        unsafe {
            neon_runtime::task::schedule(
                mem::transmute(self_raw),
                perform_task::<Self>,
                complete_task::<Self>,
                callback_raw,
            );
        }
    }
}

impl<T: Task> Worker for T {
    type Complete = <T as Task>::Complete;
    type Error = <T as Task>::Error;
    type Event = ();
    type IncomingEvent = ();
    type IncomingEventError = <T as Task>::Error;

    type JsComplete = <T as Task>::JsComplete;
    type JsEvent = JsUndefined;

    fn complete<'a, S: Scope<'a>>(
        &'a self,
        scope: &'a mut S,
        result: Result<&Self::Complete, &Self::Error>,
    ) -> JsResult<Self::JsComplete> {
        <T as Task>::complete(self, scope, result)
    }

    fn perform<N: FnMut(Message<Self::Event, Self::Error, Self::Complete>)>(
        &self,
        mut emit: N,
        _: Receiver<Self::IncomingEvent>,
    ) {
        match <T as Task>::perform(self) {
            Ok(value) => emit(Message::Complete(value)),
            Err(error) => emit(Message::Error(error)),
        }
    }

    fn event<'a, S: Scope<'a>>(&'a self, _: &'a mut S, _: &Self::Event) -> JsResult<Self::JsEvent> {
        Ok(JsUndefined::new())
    }

    fn on_incoming_event<'a>(_: Call<'a>) -> Result<Self::IncomingEvent, Self::IncomingEventError> {
        Ok(())
    }
}

unsafe extern "C" fn perform_task<T: Task>(task: *mut c_void) -> *mut c_void {
    let task: Box<T> = Box::from_raw(mem::transmute(task));
    let result = task.perform();
    Box::into_raw(task);
    mem::transmute(Box::into_raw(Box::new(result)))
}

unsafe extern "C" fn complete_task<T: Task>(
    task: *mut c_void,
    result: *mut c_void,
    out: &mut raw::Local,
) {
    let result: Result<T::Complete, T::Error> = *Box::from_raw(mem::transmute(result));
    let task: Box<T> = Box::from_raw(mem::transmute(task));

    // The neon::Task::complete() method installs an outer v8::HandleScope
    // that is responsible for managing the out pointer, so it's safe to
    // create the RootScope here without creating a local v8::HandleScope.
    let mut scope = RootScope::new(Isolate::current());
    if let Ok(result) = task.complete(&mut scope, result.as_ref()) {
        *out = result.to_raw();
    }
}
