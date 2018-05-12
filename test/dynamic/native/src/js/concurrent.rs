use std::error::Error;
use std::fmt;
use std::sync::mpsc::Receiver;

use neon::concurrent::{Message, Task, Worker};
use neon::js::error::{JsError, Kind};
use neon::js::{JsFunction, JsNumber, JsString, JsUndefined};
use neon::scope::Scope;
use neon::vm::{Call, JsResult};

#[derive(Debug)]
struct TaskError;

impl Error for TaskError {
    fn description(&self) -> &str {
        "Oops"
    }
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oops")
    }
}

#[derive(Debug)]
struct SuccessTask;

impl Task for SuccessTask {
    type Complete = i32;
    type Error = TaskError;
    type JsComplete = JsNumber;

    fn perform(&self) -> Result<Self::Complete, Self::Error> {
        Ok(10 + 7)
    }

    fn complete<'a, T: Scope<'a>>(
        &'a self,
        scope: &'a mut T,
        result: Result<&Self::Complete, &Self::Error>,
    ) -> JsResult<Self::JsComplete> {
        Ok(JsNumber::new(scope, *result.unwrap() as f64))
    }
}

#[derive(Debug)]
struct SuccessWorker;

impl Worker for SuccessWorker {
    type Complete = String;
    type Error = TaskError;
    type Event = String;
    type IncomingEvent = String;
    type IncomingEventError = TaskError;

    type JsComplete = JsString;
    type JsEvent = JsString;

    fn perform<N: FnMut(Message<Self::Event, Self::Error, Self::Complete>)>(
        &self,
        mut emit: N,
        receiver: Receiver<Self::IncomingEvent>,
    ) {
        let incoming = receiver.recv().unwrap();
        let hello = String::from("Hello");
        let world = String::from("World");

        emit(Message::Event(hello));
        emit(Message::Event(world));
        emit(Message::Complete(incoming))
    }

    fn on_incoming_event<'a>(
        call: Call<'a>,
    ) -> Result<Self::IncomingEvent, Self::IncomingEventError> {
        call.arguments
            .require(call.scope, 0)
            .and_then(|arg| arg.check::<JsString>())
            .and_then(|string| Ok(string.value()))
            .or_else(|_| Err(TaskError {}))
    }

    fn event<'a, T: Scope<'a>>(
        &'a self,
        scope: &'a mut T,
        value: &Self::Event,
    ) -> JsResult<Self::JsEvent> {
        JsString::new_or_throw(scope, value)
    }

    fn complete<'a, T: Scope<'a>>(
        &'a self,
        scope: &'a mut T,
        result: Result<&Self::Complete, &Self::Error>,
    ) -> JsResult<Self::JsComplete> {
        match result {
            Err(e) => JsError::throw(Kind::Error, e.description()),
            Ok(value) => JsString::new_or_throw(scope, value),
        }
    }
}

pub fn perform_async_task(call: Call) -> JsResult<JsUndefined> {
    let callback = call.arguments
        .require(call.scope, 0)?
        .check::<JsFunction>()?;
    let _ = SuccessTask.run(call.scope, callback);
    Ok(JsUndefined::new())
}

pub fn perform_async_task_uv(call: Call) -> JsResult<JsUndefined> {
    let callback = call.arguments
        .require(call.scope, 0)?
        .check::<JsFunction>()?;
    SuccessTask.run_uv(callback);
    Ok(JsUndefined::new())
}

pub fn create_success_worker(call: Call) -> JsResult<JsFunction> {
    let callback = call.arguments
        .require(call.scope, 0)?
        .check::<JsFunction>()?;
    SuccessWorker.spawn(call.scope, callback)
}
