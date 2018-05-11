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
    type Next = String;
    type Incoming = String;
    type IncomingError = TaskError;

    type JsComplete = JsString;
    type JsNext = JsString;

    fn perform<N: FnMut(Message<Self::Next, Self::Error, Self::Complete>)>(
        &self,
        mut emit: N,
        receiver: Receiver<Self::Incoming>,
    ) {
        let incoming = receiver.recv().unwrap();
        let hello = String::from("Hello");
        let world = String::from("World");

        emit(Message::Next(hello));
        emit(Message::Next(world));
        emit(Message::Complete(incoming))
    }

    fn on_next<'a>(call: Call<'a>) -> Result<Self::Incoming, Self::IncomingError> {
        call.arguments
            .require(call.scope, 0)
            .and_then(|arg| arg.check::<JsString>())
            .and_then(|string| Ok(string.value()))
            .or_else(|_| Err(TaskError {}))
    }

    fn next<'a, T: Scope<'a>>(
        &'a self,
        scope: &'a mut T,
        value: &Self::Next,
    ) -> JsResult<Self::JsNext> {
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
    let error = call.arguments
        .require(call.scope, 0)?
        .check::<JsFunction>()?;
    let complete = call.arguments
        .require(call.scope, 1)?
        .check::<JsFunction>()?;
    let _ = SuccessTask.run(call.scope, error, complete);
    Ok(JsUndefined::new())
}

pub fn perform_async_task_uv(call: Call) -> JsResult<JsUndefined> {
    let f = call.arguments
        .require(call.scope, 0)?
        .check::<JsFunction>()?;
    SuccessTask.run_uv(f);
    Ok(JsUndefined::new())
}

pub fn create_success_worker(call: Call) -> JsResult<JsFunction> {
    let error = call.arguments
        .require(call.scope, 0)?
        .check::<JsFunction>()?;
    let complete = call.arguments
        .require(call.scope, 1)?
        .check::<JsFunction>()?;
    let next = call.arguments
        .require(call.scope, 2)?
        .check::<JsFunction>()?;
    SuccessWorker.spawn(call.scope, error, complete, Some(next))
}
