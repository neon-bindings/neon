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

use neon_runtime::raw;
use neon_runtime::raw::Persistent;

use async::AsyncHandle;

use concurrent::crossbeam::sync::SegQueue;

#[derive(Debug)]
pub enum Message<T, E, C> {
    Next(T),
    Error(E),
    Complete(C),
}

#[derive(Debug)]
pub struct AsyncContext<T> {
    complete: Persistent,
    error: Persistent,
    next: Option<Persistent>,
    async_handle: AsyncHandle,
    queue: SegQueue<T>,
    completion_sender: Sender<()>,
    completion_receiver: Receiver<()>,
}

impl<T> AsyncContext<T> {
    fn new(
        complete: Handle<JsFunction>,
        error: Handle<JsFunction>,
        next: Option<Handle<JsFunction>>,
    ) -> Self {
        let async_handle = AsyncHandle::new();
        let queue: SegQueue<T> = SegQueue::new();
        let (completion_sender, completion_receiver): (Sender<()>, Receiver<()>) = channel();

        let next = next.and_then(|callback| Some(Persistent::new(callback.to_raw())));

        AsyncContext {
            complete: Persistent::new(complete.to_raw()),
            error: Persistent::new(error.to_raw()),
            next,
            async_handle,
            queue,
            completion_sender,
            completion_receiver,
        }
    }
}

pub trait Worker: Send + Sized + 'static {
    type Complete: Send + Sync;
    type Error: Send + Sync + Error;
    type Next: Send + Sync;
    type Incoming: Send + Sync;
    type IncomingError: Error + Sized;

    type JsComplete: Value;
    type JsNext: Value;

    fn complete<'a, T: Scope<'a>>(
        &'a self,
        scope: &'a mut T,
        result: Result<&Self::Complete, &Self::Error>,
    ) -> JsResult<Self::JsComplete>;

    fn next<'a, T: Scope<'a>>(
        &'a self,
        scope: &'a mut T,
        value: &Self::Next,
    ) -> JsResult<Self::JsNext>;

    fn on_next<'a>(call: Call<'a>) -> Result<Self::Incoming, Self::IncomingError>;

    fn perform<N: Fn(Message<Self::Next, Self::Error, Self::Complete>)>(
        &self,
        emit: N,
        receiver: Receiver<Self::Incoming>,
    );

    fn spawn<'a, T: Scope<'a>>(
        self,
        scope: &'a mut T,
        error: Handle<JsFunction>,
        complete: Handle<JsFunction>,
        next: Option<Handle<JsFunction>>,
    ) -> JsResult<'a, JsFunction> {
        let AsyncContext {
            complete,
            error,
            next,
            async_handle,
            queue,
            completion_sender,
            completion_receiver,
        } = AsyncContext::new(complete, error, next);

        let (event_sender, event_receiver): (
            Sender<Self::Incoming>,
            Receiver<Self::Incoming>,
        ) = channel();

        let _ = thread::spawn(move || {
            let callback = |value: Message<Self::Next, Self::Error, Self::Complete>| {
                // TODO: document why this is necessary (libuv coalescing uv_async_send)
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
                            Message::Next(next_value) => {
                                let result = self.next(scope, &next_value);
                                if let Some(next) = next {
                                    next.call(vec![result.unwrap().to_raw()]);
                                }
                            }
                            Message::Error(error_value) => {
                                let result = self.complete(scope, Err(&error_value));
                                error.call(vec![result.unwrap().to_raw()]);
                            }
                            Message::Complete(complete_value) => {
                                is_complete = true;
                                let result = self.complete(scope, Ok(&complete_value));
                                complete.call(vec![result.unwrap().to_raw()]);
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
                Self::on_next(inner)
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

pub trait Task: Send + Sync + Sized + 'static {
    type Complete: Send + Sync;
    type Error: Send + Sync + Error;

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
        error: Handle<JsFunction>,
        complete: Handle<JsFunction>,
    ) -> JsResult<'a, JsFunction> {
        <Self as Worker>::spawn(self, scope, error, complete, None)
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
    type Next = ();
    type Incoming = ();
    type IncomingError = <T as Task>::Error;

    type JsComplete = <T as Task>::JsComplete;
    type JsNext = JsUndefined;

    fn complete<'a, S: Scope<'a>>(
        &'a self,
        scope: &'a mut S,
        result: Result<&Self::Complete, &Self::Error>,
    ) -> JsResult<Self::JsComplete> {
        <T as Task>::complete(self, scope, result)
    }

    fn perform<N: Fn(Message<Self::Next, Self::Error, Self::Complete>)>(
        &self,
        emit: N,
        _: Receiver<Self::Incoming>,
    ) {
        match <T as Task>::perform(self) {
            Ok(value) => emit(Message::Complete(value)),
            Err(error) => emit(Message::Error(error)),
        }
    }

    fn next<'a, S: Scope<'a>>(&'a self, _: &'a mut S, _: &Self::Next) -> JsResult<Self::JsNext> {
        Ok(JsUndefined::new())
    }

    fn on_next<'a>(_: Call<'a>) -> Result<Self::Incoming, Self::IncomingError> {
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
