use neon::vm::{Call, JsResult};
use neon::scope::{Scope};
use neon::js::{JsUndefined, JsNumber, JsFunction};
use neon::js::error::{Kind, JsError};
use neon::task::Task;

struct SuccessTask;

impl Task for SuccessTask {
    type Output = i32;
    type Error = String;
    type JsEvent = JsNumber;

    fn perform(&mut self) -> Result<Self::Output, Self::Error> {
        Ok(17)
    }

    fn complete<'a, T: Scope<'a>>(self, scope: &'a mut T, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        Ok(JsNumber::new(scope, result.unwrap() as f64))
    }
}

pub fn perform_async_task(call: Call) -> JsResult<JsUndefined> {
    let f = call.arguments.require(call.scope, 0)?.check::<JsFunction>()?;
    SuccessTask.schedule(f);
    Ok(JsUndefined::new())
}

struct FailureTask;

impl Task for FailureTask {
    type Output = i32;
    type Error = String;
    type JsEvent = JsNumber;

    fn perform(&mut self) -> Result<Self::Output, Self::Error> {
        Err(format!("I am a failing task"))
    }

    fn complete<'a, T: Scope<'a>>(self, _: &'a mut T, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        JsError::throw(Kind::Error, &result.unwrap_err())
    }
}

pub fn perform_failing_task(call: Call) -> JsResult<JsUndefined> {
    let f = call.arguments.require(call.scope, 0)?.check::<JsFunction>()?;
    FailureTask.schedule(f);
    Ok(JsUndefined::new())
}
