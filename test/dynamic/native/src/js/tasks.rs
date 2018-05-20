// FIXME: should FunctionContext live in js?
use neon::vm::{FunctionContext, JsResult, Context};
use neon::js::{JsUndefined, JsNumber, JsFunction};
use neon::js::error::{Kind, JsError};
// FIXME: should TaskContext live in vm?
use neon::task::{Task, TaskContext};

struct SuccessTask;

impl Task for SuccessTask {
    type Output = i32;
    type Error = String;
    type JsEvent = JsNumber;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        Ok(17)
    }

    fn complete(self, mut cx: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        Ok(cx.number(result.unwrap()))
    }
}

pub fn perform_async_task(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let f = cx.argument::<JsFunction>(0)?;
    SuccessTask.schedule(f);
    Ok(cx.undefined())
}

struct FailureTask;

impl Task for FailureTask {
    type Output = i32;
    type Error = String;
    type JsEvent = JsNumber;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        Err(format!("I am a failing task"))
    }

    fn complete(self, _: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        // FIXME: does throw need an &mut context for soundness?
        JsError::throw(Kind::Error, &result.unwrap_err())
    }
}

pub fn perform_failing_task(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let f = cx.argument::<JsFunction>(0)?;
    FailureTask.schedule(f);
    Ok(cx.undefined())
}
