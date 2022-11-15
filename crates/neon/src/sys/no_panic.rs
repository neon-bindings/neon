//! Utilities that _will_ not panic for use in contexts where unwinding would be
//! undefined behavior.
//!
//! The following helpers do not panic and instead use `napi_fatal_error`
//! to crash the process in a controlled way, making them safe for use in FFI
//! callbacks.
//!
//! `#[track_caller]` is used on these helpers to ensure `fatal_error` reports
//! the calling location instead of the helpers defined here.

use std::{
    any::Any,
    ffi::c_void,
    mem::MaybeUninit,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
};

use super::{
    bindings as napi,
    error::fatal_error,
    raw::{Env, Local},
};

type Panic = Box<dyn Any + Send + 'static>;

const UNKNOWN_PANIC_MESSAGE: &str = "Unknown panic";

/// `FailureBoundary`] acts as boundary between Rust and FFI code, protecting
/// a critical section of code from unhandled failure. It will catch both Rust
/// panics and JavaScript exceptions. Attempts to handle failures are executed
/// in order of ascending severity:
///
/// 1. Reject a `Promise` if a `Deferred` was provided
/// 2. Emit a fatal exception
/// 3. Abort the process with a message and location
///
/// This process will be aborted if any step unrecoverably fails. For example,
/// if a `napi::Env` is unavailable, it is impossible to reject a `Promise` or
/// emit a fatal exception.
pub struct FailureBoundary {
    pub both: &'static str,
    pub exception: &'static str,
    pub panic: &'static str,
}

impl FailureBoundary {
    #[track_caller]
    pub unsafe fn catch_failure<F>(&self, env: Env, deferred: Option<napi::Deferred>, f: F)
    where
        F: FnOnce(Option<Env>) -> Local,
    {
        // Make `env = None` if unable to call into JS
        #[allow(clippy::unnecessary_lazy_evaluations)]
        let env = can_call_into_js(env).then(|| env);

        // Run the user supplied callback, catching panics
        // This is unwind safe because control is never yielded back to the caller
        let panic = catch_unwind(AssertUnwindSafe(move || f(env)));

        // Unwrap the `Env`
        let env = if let Some(env) = env {
            env
        } else {
            // If there was a panic and we don't have an `Env`, crash the process
            if let Err(panic) = panic {
                let msg = panic_msg(&panic).unwrap_or(UNKNOWN_PANIC_MESSAGE);

                fatal_error(msg);
            }

            // If we don't have an `Env`, we can't catch an exception, nothing more to try
            return;
        };

        // Check and catch a thrown exception
        let exception = catch_exception(env);

        // Create an error message or return if there wasn't a panic or exception
        let msg = match (exception, panic.as_ref()) {
            // Exception and a panic
            (Some(_), Err(_)) => self.both,

            // Exception, but not a panic
            (Some(err), Ok(_)) => {
                // Reject the promise without wrapping
                if let Some(deferred) = deferred {
                    reject_deferred(env, deferred, err);

                    return;
                }

                self.exception
            }

            // Panic, but not an exception
            (None, Err(_)) => self.panic,

            // No errors occurred! We're done!
            (None, Ok(value)) => {
                if let Some(deferred) = deferred {
                    resolve_deferred(env, deferred, *value);
                }

                return;
            }
        };

        // Reject the promise
        if let Some(deferred) = deferred {
            let error = create_error(env, msg, exception, panic.err());

            reject_deferred(env, deferred, error);

            return;
        }

        let error = create_error(env, msg, exception, panic.err());

        // Trigger a fatal exception
        fatal_exception(env, error);
    }
}

// HACK: Force `NAPI_PREAMBLE` to run without executing any JavaScript to tell if it's
// possible to call into JS.
//
// `NAPI_PREAMBLE` is a macro that checks if it is possible to call into JS.
// https://github.com/nodejs/node/blob/5fad0b93667ffc6e4def52996b9529ac99b26319/src/js_native_api_v8.h#L211-L218
//
//  `napi_throw` starts by using `NAPI_PREAMBLE` and then a `CHECK_ARGS` on the `napi_value`. Since
// we already know `env` is non-null, we expect the `null` value to cause a `napi_invalid_arg` error.
// https://github.com/nodejs/node/blob/5fad0b93667ffc6e4def52996b9529ac99b26319/src/js_native_api_v8.cc#L1925-L1926
fn can_call_into_js(env: Env) -> bool {
    !env.is_null() && unsafe { napi::throw(env, ptr::null_mut()) == napi::Status::InvalidArg }
}

// We cannot use `napi_fatal_exception` because of this bug; instead, cause an
// unhandled rejection which has similar behavior on recent versions of Node.
// https://github.com/nodejs/node/issues/33771
unsafe fn fatal_exception(env: Env, error: Local) {
    let mut deferred = MaybeUninit::uninit();
    let mut promise = MaybeUninit::uninit();

    let deferred = match napi::create_promise(env, deferred.as_mut_ptr(), promise.as_mut_ptr()) {
        napi::Status::Ok => deferred.assume_init(),
        _ => fatal_error("Failed to create a promise"),
    };

    if napi::reject_deferred(env, deferred, error) != napi::Status::Ok {
        fatal_error("Failed to reject a promise");
    }
}

#[track_caller]
unsafe fn create_error(
    env: Env,
    msg: &str,
    exception: Option<Local>,
    panic: Option<Panic>,
) -> Local {
    // Construct the `uncaughtException` Error object
    let error = error_from_message(env, msg);

    // Add the exception to the error
    if let Some(exception) = exception {
        set_property(env, error, "cause", exception);
    };

    // Add the panic to the error
    if let Some(panic) = panic {
        set_property(env, error, "panic", error_from_panic(env, panic));
    }

    error
}

#[track_caller]
unsafe fn resolve_deferred(env: Env, deferred: napi::Deferred, value: Local) {
    if napi::resolve_deferred(env, deferred, value) != napi::Status::Ok {
        fatal_error("Failed to resolve promise");
    }
}

#[track_caller]
unsafe fn reject_deferred(env: Env, deferred: napi::Deferred, value: Local) {
    if napi::reject_deferred(env, deferred, value) != napi::Status::Ok {
        fatal_error("Failed to reject promise");
    }
}

#[track_caller]
unsafe fn catch_exception(env: Env) -> Option<Local> {
    if !is_exception_pending(env) {
        return None;
    }

    let mut error = MaybeUninit::uninit();

    if napi::get_and_clear_last_exception(env, error.as_mut_ptr()) != napi::Status::Ok {
        fatal_error("Failed to get and clear the last exception");
    }

    Some(error.assume_init())
}

#[track_caller]
unsafe fn error_from_message(env: Env, msg: &str) -> Local {
    let msg = create_string(env, msg);
    let mut err = MaybeUninit::uninit();

    let status = napi::create_error(env, ptr::null_mut(), msg, err.as_mut_ptr());

    if status == napi::Status::Ok {
        err.assume_init()
    } else {
        fatal_error("Failed to create an Error");
    }
}

#[track_caller]
unsafe fn error_from_panic(env: Env, panic: Panic) -> Local {
    if let Some(msg) = panic_msg(&panic) {
        error_from_message(env, msg)
    } else {
        let error = error_from_message(env, UNKNOWN_PANIC_MESSAGE);
        let panic = external_from_panic(env, panic);

        set_property(env, error, "cause", panic);
        error
    }
}

#[track_caller]
unsafe fn set_property(env: Env, object: Local, key: &str, value: Local) {
    let key = create_string(env, key);

    if napi::set_property(env, object, key, value) != napi::Status::Ok {
        fatal_error("Failed to set an object property");
    }
}

#[track_caller]
unsafe fn panic_msg(panic: &Panic) -> Option<&str> {
    if let Some(msg) = panic.downcast_ref::<&str>() {
        Some(msg)
    } else if let Some(msg) = panic.downcast_ref::<String>() {
        Some(msg)
    } else {
        None
    }
}

unsafe fn external_from_panic(env: Env, panic: Panic) -> Local {
    let fail = || fatal_error("Failed to create a neon::types::JsBox from a panic");
    let mut result = MaybeUninit::uninit();
    let status = napi::create_external(
        env,
        Box::into_raw(Box::new(panic)).cast(),
        Some(finalize_panic),
        ptr::null_mut(),
        result.as_mut_ptr(),
    );

    if status != napi::Status::Ok {
        fail();
    }

    let external = result.assume_init();

    #[cfg(feature = "napi-8")]
    if napi::type_tag_object(env, external, &*crate::MODULE_TAG) != napi::Status::Ok {
        fail();
    }

    external
}

extern "C" fn finalize_panic(_env: Env, data: *mut c_void, _hint: *mut c_void) {
    unsafe {
        drop(Box::from_raw(data.cast::<Panic>()));
    }
}

#[track_caller]
unsafe fn create_string(env: Env, msg: &str) -> Local {
    let mut string = MaybeUninit::uninit();
    let status = napi::create_string_utf8(env, msg.as_ptr().cast(), msg.len(), string.as_mut_ptr());

    if status != napi::Status::Ok {
        fatal_error("Failed to create a String");
    }

    string.assume_init()
}

unsafe fn is_exception_pending(env: Env) -> bool {
    let mut throwing = false;

    if napi::is_exception_pending(env, &mut throwing) != napi::Status::Ok {
        fatal_error("Failed to check if an exception is pending");
    }

    throwing
}
