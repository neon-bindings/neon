//! Rust wrappers for Node-API simple asynchronous operations
//!
//! Unlike `napi_async_work` which threads a single mutable pointer to a data
//! struct to both the `execute` and `complete` callbacks, the wrapper follows
//! a more idiomatic Rust ownership pattern by passing the output of `execute`
//! into the input of `complete`.
//!
//! https://nodejs.org/api/n-api.html#n_api_simple_asynchronous_operations

use std::{
    ffi::c_void,
    mem,
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    ptr, thread,
};

use super::{bindings as napi, no_panic::FailureBoundary, raw::Env};

const BOUNDARY: FailureBoundary = FailureBoundary {
    both: "A panic and exception occurred while executing a `neon::event::TaskBuilder` task",
    exception: "An exception occurred while executing a `neon::event::TaskBuilder` task",
    panic: "A panic occurred while executing a `neon::event::TaskBuilder` task",
};

type Execute<I, O> = fn(input: I) -> O;
type Complete<O, D> = fn(env: Env, output: thread::Result<O>, data: D);

/// Schedule work to execute on the libuv thread pool
///
/// # Safety
/// * `env` must be a valid `napi_env` for the current thread
/// * The `thread::Result::Err` must only be used for resuming unwind if
///   `execute` is not unwind safe
pub unsafe fn schedule<I, O, D>(
    env: Env,
    input: I,
    execute: Execute<I, O>,
    complete: Complete<O, D>,
    data: D,
) where
    I: Send + 'static,
    O: Send + 'static,
    D: Send + 'static,
{
    let mut data = Box::new(Data {
        state: State::Input(input),
        execute,
        complete,
        data,
        // Work is initialized as a null pointer, but set by `create_async_work`
        // `data` must not be used until this value has been set.
        work: ptr::null_mut(),
    });

    // Store a pointer to `work` before ownership is transferred to `Box::into_raw`
    let work = &mut data.work as *mut _;

    // Create the `async_work`
    assert_eq!(
        napi::create_async_work(
            env,
            ptr::null_mut(),
            super::string(env, "neon_async_work"),
            Some(call_execute::<I, O, D>),
            Some(call_complete::<I, O, D>),
            Box::into_raw(data).cast(),
            work,
        ),
        napi::Status::Ok,
    );

    // Queue the work
    match napi::queue_async_work(env, *work) {
        napi::Status::Ok => {}
        status => {
            // If queueing failed, delete the work to prevent a leak
            napi::delete_async_work(env, *work);
            assert_eq!(status, napi::Status::Ok);
        }
    }
}

/// A pointer to data is passed to the `execute` and `complete` callbacks
struct Data<I, O, D> {
    state: State<I, O>,
    execute: Execute<I, O>,
    complete: Complete<O, D>,
    data: D,
    work: napi::AsyncWork,
}

/// State of the task that is transitioned by `execute` and `complete`
enum State<I, O> {
    /// Initial data input passed to `execute`
    Input(I),
    /// Transient state while `execute` is running
    Executing,
    /// Return data of `execute` passed to `complete`
    Output(thread::Result<O>),
}

impl<I, O> State<I, O> {
    /// Return the input if `State::Input`, replacing with `State::Executing`
    fn take_execute_input(&mut self) -> Option<I> {
        match mem::replace(self, Self::Executing) {
            Self::Input(input) => Some(input),
            _ => None,
        }
    }

    /// Return the output if `State::Output`, replacing with `State::Executing`
    fn into_output(self) -> Option<thread::Result<O>> {
        match self {
            Self::Output(output) => Some(output),
            _ => None,
        }
    }
}

/// Callback executed on the libuv thread pool
///
/// # Safety
/// * `Env` should not be used because it could attempt to call JavaScript
/// * `data` is expected to be a pointer to `Data<I, O, D>`
unsafe extern "C" fn call_execute<I, O, D>(_: Env, data: *mut c_void) {
    let data = &mut *data.cast::<Data<I, O, D>>();

    // This is unwind safe because unwinding will resume on the other side
    let output = catch_unwind(AssertUnwindSafe(|| {
        // `unwrap` is ok because `call_execute` should be called exactly once
        // after initialization
        let input = data.state.take_execute_input().unwrap();

        (data.execute)(input)
    }));

    data.state = State::Output(output);
}

/// Callback executed on the JavaScript main thread
///
/// # Safety
/// * `data` is expected to be a pointer to `Data<I, O, D>`
unsafe extern "C" fn call_complete<I, O, D>(env: Env, status: napi::Status, data: *mut c_void) {
    let Data {
        state,
        complete,
        data,
        work,
        ..
    } = *Box::<Data<I, O, D>>::from_raw(data.cast());

    napi::delete_async_work(env, work);

    BOUNDARY.catch_failure(env, None, move |env| {
        // `unwrap` is okay because `call_complete` should be called exactly once
        // if and only if `call_execute` has completed successfully
        let output = state.into_output().unwrap();

        // The event looped has stopped if we do not have an Env
        let env = if let Some(env) = env {
            env
        } else {
            // Resume panicking if necessary
            if let Err(panic) = output {
                resume_unwind(panic);
            }

            return ptr::null_mut();
        };

        match status {
            napi::Status::Ok => complete(env, output, data),
            napi::Status::Cancelled => {}
            _ => assert_eq!(status, napi::Status::Ok),
        }

        ptr::null_mut()
    });
}
