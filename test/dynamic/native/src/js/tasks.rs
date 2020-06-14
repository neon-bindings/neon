use neon::prelude::*;

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

pub fn perform_closure_task(mut cx: FunctionContext) -> JsResult<JsValue> {
    let n = cx.argument::<JsNumber>(0)?.value();
    let cb = cx.argument::<JsFunction>(1)?;

    cx.task(move || {
            let result = n + 1.0;

            move |mut cx| Ok(cx.number(result))
        })
        .schedule(cb)
}

struct FailureTask;

impl Task for FailureTask {
    type Output = i32;
    type Error = String;
    type JsEvent = JsNumber;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        Err(format!("I am a failing task"))
    }

    fn complete(self, mut cx: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        cx.throw_error(&result.unwrap_err())
    }
}

pub fn perform_failing_task(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let f = cx.argument::<JsFunction>(0)?;
    FailureTask.schedule(f);
    Ok(cx.undefined())
}
