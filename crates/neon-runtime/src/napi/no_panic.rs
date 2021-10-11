//! Utilities that _will_ not panic for use in contexts where unwinding would be
//! undefined behavior.
//!
//! The following helpers do not panic and instead use `napi_fatal_error`
//! to crash the process in a controlled way, making them safe for use in FFI
//! callbacks.
//!
//! `#[track_caller]` is used on these helpers to ensure `fatal_error` reports
//! the calling location instead of the helpers defined here.

use std::any::Any;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use super::bindings as napi;
use super::error::fatal_error;
use super::raw::{Env, Local};

type Panic = Box<dyn Any + Send + 'static>;

const UNKNOWN_PANIC_MESSAGE: &str = "Unknown panic";

pub(super) struct ExceptionPanicHandler {
    pub both: &'static str,
    pub exception: &'static str,
    pub panic: &'static str,
}

impl ExceptionPanicHandler {
    #[track_caller]
    pub(super) unsafe fn handle(&self, env: Env, f: impl FnOnce(Option<Env>)) {
        // Event loop has terminated if `null`
        let env = if env.is_null() { None } else { Some(env) };

        // Run the user supplied callback, catching panics
        // This is unwind safe because control is never yielded back to the caller
        let panic = catch_unwind(AssertUnwindSafe(move || f(env))).err();

        // Unwrap the `Env`
        let env = if let Some(env) = env {
            env
        } else {
            // If there was a panic and we don't have an `Env`, crash the process
            if let Some(panic) = panic {
                let msg = panic_msg(&panic).unwrap_or(UNKNOWN_PANIC_MESSAGE);

                fatal_error(msg);
            }

            // If we don't have an `Env`, we can't catch an exception, nothing more to try
            return;
        };

        // Check and catch a thrown exception
        let exception = catch_exception(env);

        // Create an error message or return if there wasn't a panic or exception
        let msg = match (exception.is_some(), panic.is_some()) {
            (true, true) => self.both,
            (true, false) => self.exception,
            (false, true) => self.panic,
            // No errors occurred! We're done!
            (false, false) => return,
        };

        #[cfg(not(feature = "napi-3"))]
        // Crash the process on Node-API < 3
        {
            let msg = panic
                .as_ref()
                .and_then(|panic| panic_msg(&panic))
                .unwrap_or(msg);

            fatal_error(msg);
        }

        #[cfg(feature = "napi-3")]
        // Throw an `uncaughtException` on Node-API >= 3
        {
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

            // Throw an uncaught exception
            if napi::fatal_exception(env, error) != napi::Status::Ok {
                fatal_error("Failed to throw an uncaughtException");
            }
        }
    }
}

#[track_caller]
pub unsafe fn create_panic_error(env: Env, msg: &str, panic: Panic) -> Local {
    // Construct the Error object
    let error = error_from_message(env, msg);

    // Add the panic to the error
    set_property(env, error, "panic", error_from_panic(env, panic));

    error
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

    let err = if status == napi::Status::Ok {
        err.assume_init()
    } else {
        fatal_error("Failed to create an Error");
    };

    err
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
        Some(&msg)
    } else {
        None
    }
}

unsafe fn external_from_panic(env: Env, panic: Panic) -> Local {
    let mut result = MaybeUninit::uninit();
    let status = napi::create_external(
        env,
        Box::into_raw(Box::new(panic)).cast(),
        Some(finalize_panic),
        ptr::null_mut(),
        result.as_mut_ptr(),
    );

    if status != napi::Status::Ok {
        fatal_error("Failed to create a neon::types::JsBox from a panic");
    }

    result.assume_init()
}

extern "C" fn finalize_panic(_env: Env, data: *mut c_void, _hint: *mut c_void) {
    unsafe {
        Box::from_raw(data.cast::<Panic>());
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
