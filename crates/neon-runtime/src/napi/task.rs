use std::mem::MaybeUninit;
use std::os::raw::c_void;

use nodejs_sys as napi;

use raw::{Env, Local};

unsafe fn null(env: napi::napi_env) -> Local {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_get_null(env, result.as_mut_ptr()),
        napi::napi_status::napi_ok,
    );

    result.assume_init()
}

unsafe fn undefined(env: napi::napi_env) -> Local {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_get_undefined(env, result.as_mut_ptr()),
        napi::napi_status::napi_ok,
    );

    result.assume_init()
}

unsafe fn global(env: napi::napi_env) -> Local {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_get_global(env, result.as_mut_ptr()),
        napi::napi_status::napi_ok,
    );

    result.assume_init()
}

// Resource name to identify async work in diagnostic information
unsafe fn async_work_name(
    env: napi::napi_env,
) -> Local {
    let name = b"neon task";
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_create_string_utf8(
        env,
        name.as_ptr() as *const _,
        name.len(),
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::napi_status::napi_ok);

    result.assume_init()
}

// Stores state for async work being executed
struct AsyncTask<Data, Execute, Output, Complete> {
    data: Option<Data>,
    execute: Option<Execute>,
    output: Option<Output>,
    complete: Option<Complete>,
    callback: napi::napi_ref,
    work: MaybeUninit<napi::napi_async_work>,
}

// RAII guard to drop callback
struct NapiCallback {
    env: napi::napi_env,
    value: napi::napi_ref,
}

impl NapiCallback {
    unsafe fn new(
        env: napi::napi_env,
        value: Local,
    ) -> Self {
        // Create a persistent reference to the JavaScript function
        let mut result = MaybeUninit::uninit();
        let status = napi::napi_create_reference(
            env,
            value,
            1,
            result.as_mut_ptr(),
        );

        assert_eq!(status, napi::napi_status::napi_ok);

        Self {
            env,
            value: result.assume_init(),
        }
    }

    unsafe fn as_raw(&self) -> napi::napi_ref {
        self.value
    }

    unsafe fn from_raw(env: napi::napi_env, value: napi::napi_ref) -> Self {
        Self {
            env,
            value,
        }
    }
}

impl Drop for NapiCallback {
    fn drop(&mut self) {
        unsafe {
            napi::napi_delete_reference(self.env, self.value);
        }
    }
}

// RAII guard to drop async work and callback
struct NapiAsyncWork<Data, Execute, Output, Complete> {
    env: napi::napi_env,
    callback: NapiCallback,
    task: Box<AsyncTask<Data, Execute, Output, Complete>>,
    work: napi::napi_async_work,
}

impl<Data, Execute, Output, Complete> NapiAsyncWork<Data, Execute, Output, Complete>
where
    Execute: FnOnce(Data) -> Output,
    Complete: FnOnce(napi::napi_env, Output) -> Option<Local>,
{
    unsafe fn new(
        env: napi::napi_env,
        data: Data,
        execute: Execute,
        complete: Complete,
        callback: NapiCallback,
    ) -> Self {
        let name = async_work_name(env);

        // Create struct to hold state for async work
        // WARN: `AsyncWork` is self-referential. It becomes the `data` pointer
        // in the async work, which is then assigned to `work`.
        let mut task = Box::new(AsyncTask {
            data: Some(data),
            execute: Some(execute),
            output: None,
            complete: Some(complete),
            callback: callback.as_raw(),
            // `null` is a valid value and checked in N-API
            work: MaybeUninit::zeroed(),
        });

        let status = napi::napi_create_async_work(
            env,
            std::ptr::null_mut(),
            name,
            Some(perform_work::<Data, Execute, Output, Complete>),
            Some(complete_work::<Data, Execute, Output, Complete>),
            // Does not use `Box::into_raw` because `Task` might not be used
            // 1. `Task` should be dropped if the work fails to be queued
            // 2. `work` is self referential and needs to be aliased
            &mut *task as *mut _ as *mut c_void,
            task.work.as_mut_ptr(),
        );

        assert_eq!(status, napi::napi_status::napi_ok);

        let work = task.work.assume_init();

        Self {
            env,
            callback,
            task,
            work,
        }
    }

    unsafe fn as_raw(&self) -> napi::napi_async_work {
        self.work
    }

    unsafe fn from_raw(env: napi::napi_env, task: *mut c_void) -> Self {
        let task: Box<AsyncTask<Data, Execute, Output, Complete>> =
            Box::from_raw(task as *mut _);

        let work = task.work.assume_init();

        Self {
            env,
            callback: NapiCallback::from_raw(env, task.callback),
            task,
            work,
        }
    }
}

impl<Data, Execute, Output, Complete> Drop for NapiAsyncWork<Data, Execute, Output, Complete> {
    fn drop(&mut self) {
        unsafe {
            napi::napi_delete_async_work(self.env, self.work);
        }
    }
}

unsafe extern "C" fn perform_work<Data, Execute, Output, Complete>(
    _: napi::napi_env,
    task: *mut c_void,
)
where
    Execute: FnOnce(Data) -> Output,
{
    let task = &mut *(task as *mut AsyncTask<Data, Execute, Output, Complete>);
    let execute = task.execute.take().unwrap();
    let data = task.data.take().unwrap();

    task.output.replace(execute(data));
}

// Complete async work on the main thread
unsafe extern "C" fn complete_work<Data, Execute, Output, Complete>(
    env: napi::napi_env,
    status: napi::napi_status,
    task: *mut c_void,
)
where
    Execute: FnOnce(Data) -> Output,
    Complete: FnOnce(napi::napi_env, Output) -> Option<Local>,
{
    // Unbox the task first to ensure it is dropped
    let mut task = NapiAsyncWork::<Data, Execute, Output, Complete>::from_raw(env, task);

    // Neon does not support cancelling tasks but there may be other ways for
    // tasks to become cancelled.
    // Must come after `task` is unboxed to prevent memory leaks.
    if status == napi::napi_status::napi_cancelled {
        return;
    }

    // Run the provided `complete` function
    let output = task.task.output.take().unwrap();
    let complete = task.task.complete.take().unwrap();
    let output = complete(env, output);

    // Determine if an exception has been thrown
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_is_exception_pending(env, result.as_mut_ptr());
    let is_exception = {
        assert_eq!(status, napi::napi_status::napi_ok);

        result.assume_init()
    };

    // An exception has been thrown, call with `callback(err)`
    let mut argv = if is_exception {
        let mut result = MaybeUninit::uninit();
        let status = napi::napi_get_and_clear_last_exception(
            env,
            result.as_mut_ptr(),
        );

        let exception = {
            assert_eq!(status, napi::napi_status::napi_ok);

            result.assume_init()
        };

        [exception, undefined(env)]

    // Completed successfully, call with `callback(null, output)`
    } else {
        [null(env), output.unwrap_or_else(|| undefined(env))]
    };

    let mut result = MaybeUninit::uninit();
    let status = napi::napi_get_reference_value(
        env,
        task.callback.as_raw(),
        result.as_mut_ptr(),
    );

    let callback = {
        assert_eq!(status, napi::napi_status::napi_ok);

        result.assume_init()
    };

    assert_eq!(
        napi::napi_call_function(
            env,
            global(env),
            callback,
            2,
            argv.as_mut_ptr(),
            MaybeUninit::uninit().as_mut_ptr(),
        ),
        napi::napi_status::napi_ok,
    );
}

pub unsafe fn schedule<Data, Execute, Output, Complete>(
    env: Env,
    data: Data,
    execute: Execute,
    complete: Complete,
    callback: Local,
)
where
    Data: Send + Sized + 'static,
    Output: Send + Sized + 'static,
    Execute: FnOnce(Data) -> Output + Send + 'static,
    Complete: FnOnce(napi::napi_env, Output) -> Option<Local>,
{
    let callback = NapiCallback::new(env, callback);
    let task = NapiAsyncWork::<_, _, Output, _>::new(
        env,
        data,
        execute,
        complete,
        callback,
    );

    // Assumption: The queued task will execute if and only if the call to
    // queue the work returns `napi_status::napi_ok`. If it does not and the
    // task executes, a double free will occur.
    assert_eq!(
        napi::napi_queue_async_work(env, task.as_raw()),
        napi::napi_status::napi_ok,
    );

    // If we queued successfully, prevent `Drop`
    // It will be dropped in `complete`
    std::mem::forget(task);
}
