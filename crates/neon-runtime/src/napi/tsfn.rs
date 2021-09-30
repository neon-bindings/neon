//! Idiomatic Rust wrappers for N-API threadsafe functions

use std::any::Any;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::{Arc, Mutex};

use crate::napi::bindings as napi;
use crate::napi::error::fatal_error;
use crate::raw::{Env, Local};

type Panic = Box<dyn Any + Send + 'static>;

const UNKNOWN_PANIC_MESSAGE: &str = "Unknown panic";

#[derive(Debug)]
struct Tsfn(napi::ThreadsafeFunction);

unsafe impl Send for Tsfn {}

unsafe impl Sync for Tsfn {}

#[derive(Debug)]
/// Threadsafe Function encapsulate a Rust function pointer and N-API threadsafe
/// function for scheduling tasks to execute on a JavaScript thread.
pub struct ThreadsafeFunction<T> {
    tsfn: Tsfn,
    is_finalized: Arc<Mutex<bool>>,
    callback: fn(Option<Env>, T),
}

#[derive(Debug)]
struct Callback<T> {
    callback: fn(Option<Env>, T),
    data: T,
}

/// Error returned when scheduling a threadsafe function with some data
pub struct CallError<T> {
    kind: napi::Status,
    data: T,
}

impl<T> CallError<T> {
    /// The specific error that occurred
    pub fn kind(&self) -> napi::Status {
        self.kind
    }

    /// Returns the data that was sent when scheduling to allow re-scheduling
    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T: Send + 'static> ThreadsafeFunction<T> {
    /// Creates a new unbounded N-API Threadsafe Function
    /// Safety: `Env` must be valid for the current thread
    pub unsafe fn new(env: Env, callback: fn(Option<Env>, T)) -> Self {
        Self::with_capacity(env, 0, callback)
    }

    /// Creates a bounded N-API Threadsafe Function
    /// Safety: `Env` must be valid for the current thread
    pub unsafe fn with_capacity(
        env: Env,
        max_queue_size: usize,
        callback: fn(Option<Env>, T),
    ) -> Self {
        let mut result = MaybeUninit::uninit();
        let is_finalized = Arc::new(Mutex::new(false));

        assert_eq!(
            napi::create_threadsafe_function(
                env,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                super::string(env, "neon threadsafe function"),
                max_queue_size,
                // Always set the reference count to 1. Prefer using
                // Rust `Arc` to maintain the struct.
                1,
                Arc::into_raw(is_finalized.clone()) as *mut _,
                Some(Self::finalize),
                std::ptr::null_mut(),
                Some(Self::callback),
                result.as_mut_ptr(),
            ),
            napi::Status::Ok,
        );

        Self {
            tsfn: Tsfn(result.assume_init()),
            is_finalized: is_finalized,
            callback,
        }
    }

    /// Schedule a threadsafe function to be executed with some data
    pub fn call(
        &self,
        data: T,
        is_blocking: Option<napi::ThreadsafeFunctionCallMode>,
    ) -> Result<(), CallError<T>> {
        let is_blocking = is_blocking.unwrap_or(napi::ThreadsafeFunctionCallMode::Blocking);

        let callback = Box::into_raw(Box::new(Callback {
            callback: self.callback,
            data,
        }));

        // Hold the lock before entering `call_threadsafe_function` so that
        // `finalize_cb` would never complete.
        let mut is_finalized = self.is_finalized.lock().unwrap();

        let status = {
            if *is_finalized {
                napi::Status::Closing
            } else {
                unsafe {
                    napi::call_threadsafe_function(self.tsfn.0, callback as *mut _, is_blocking)
                }
            }
        };

        if status == napi::Status::Ok {
            Ok(())
        } else {
            // Prevent further calls to `call_threadsafe_function`
            if status == napi::Status::Closing {
                *is_finalized = true;
            }

            // If the call failed, the callback won't execute
            let callback = unsafe { Box::from_raw(callback) };

            Err(CallError {
                kind: status,
                data: callback.data,
            })
        }
    }

    /// References a threadsafe function to prevent exiting the event loop until it has been dropped. (Default)
    /// Safety: `Env` must be valid for the current thread
    pub unsafe fn reference(&self, env: Env) {
        assert_eq!(
            napi::ref_threadsafe_function(env, self.tsfn.0),
            napi::Status::Ok,
        );
    }

    /// Unreferences a threadsafe function to allow exiting the event loop before it has been dropped.
    /// Safety: `Env` must be valid for the current thread
    pub unsafe fn unref(&self, env: Env) {
        assert_eq!(
            napi::unref_threadsafe_function(env, self.tsfn.0),
            napi::Status::Ok,
        );
    }

    // Provides a C ABI wrapper for a napi callback notifying us about tsfn
    // being finalized.
    unsafe extern "C" fn finalize(_env: Env, data: *mut c_void, _hint: *mut c_void) {
        let is_finalized = Arc::from_raw(data as *mut Mutex<bool>);

        *is_finalized.lock().unwrap() = true;
    }

    // Provides a C ABI wrapper for invoking the user supplied function pointer
    // On panic or exception, creates an `uncaughtException` of the form:
    // Error(msg: string) {
    //     // Exception thrown
    //     cause?: Error,
    //     // Panic occurred
    //     panic?: Error(msg: string) {
    //         // Opaque panic type if it wasn't a string
    //         cause?: JsBox<Panic>
    //     }
    // }
    unsafe extern "C" fn callback(
        env: Env,
        _js_callback: napi::Value,
        _context: *mut c_void,
        data: *mut c_void,
    ) {
        let Callback { callback, data } = *Box::from_raw(data as *mut Callback<T>);

        // Event loop has terminated if `null`
        let env = if env.is_null() { None } else { Some(env) };

        // Run the user supplied callback, catching panics
        // This is unwind safe because control is never yielded back to the caller
        let panic = catch_unwind(AssertUnwindSafe(move || callback(env, data))).err();

        let env = if let Some(env) = env {
            env
        } else {
            // If we don't have an Env, at most we can print a panic message
            if let Some(panic) = panic {
                let msg = no_panic::panic_msg(&panic).unwrap_or(UNKNOWN_PANIC_MESSAGE);

                eprintln!("{}", msg);
            }

            return;
        };

        // Check and catch a thrown exception
        let exception = no_panic::catch_exception(env);

        // Create an error message or return if there wasn't a panic or exception
        let msg = match (panic.is_some(), exception.is_some()) {
            (true, true) => "A panic and exception occurred while executing a `neon::event::Channel::send` callback",
            (true, false) => "A panic occurred while executing a `neon::event::Channel::send` callback",
            (false, true) => "An exception occurred while executing a `neon::event::Channel::send` callback",
            (false, false) => return
        };

        // Construct the `uncaughtException` Error object
        let error = no_panic::error_from_message(env, msg);

        // Add the exception to the error
        if let Some(exception) = exception {
            no_panic::set_property(env, error, "cause", exception);
        };

        // Add the panic to the error
        if let Some(panic) = panic {
            no_panic::set_property(env, error, "panic", no_panic::error_from_panic(env, panic));
        }

        // Throw an uncaught exception
        if napi::fatal_exception(env, error) != napi::Status::Ok {
            fatal_error("Failed to throw an uncaughtException");
        }
    }
}

impl<T> Drop for ThreadsafeFunction<T> {
    fn drop(&mut self) {
        let is_finalized = self.is_finalized.lock().unwrap();

        // tsfn was already finalized by `Environment::CleanupHandles()` in Node.js
        if *is_finalized {
            return;
        }

        unsafe {
            napi::release_threadsafe_function(
                self.tsfn.0,
                napi::ThreadsafeFunctionReleaseMode::Release,
            );
        };
    }
}

// The following helpers do not panic and instead use `napi_fatal_error` to crash the
// process in a controlled way, making them safe for use in FFI callbacks.
//
// `#[track_caller]` is used on these helpers to ensure `fatal_error` reports the calling
// location instead of the helpers defined here.
mod no_panic {
    use super::*;

    #[track_caller]
    pub(super) unsafe fn catch_exception(env: Env) -> Option<Local> {
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
    pub(super) unsafe fn error_from_message(env: Env, msg: &str) -> Local {
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
    pub(super) unsafe fn error_from_panic(env: Env, panic: Panic) -> Local {
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
    pub(super) unsafe fn set_property(env: Env, object: Local, key: &str, value: Local) {
        let key = create_string(env, key);

        if napi::set_property(env, object, key, value) != napi::Status::Ok {
            fatal_error("Failed to set an object property");
        }
    }

    #[track_caller]
    pub(super) unsafe fn panic_msg(panic: &Panic) -> Option<&str> {
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
        let status =
            napi::create_string_utf8(env, msg.as_ptr().cast(), msg.len(), string.as_mut_ptr());

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
}
