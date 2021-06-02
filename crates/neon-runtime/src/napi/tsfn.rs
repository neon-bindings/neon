//! Idiomatic Rust wrappers for N-API threadsafe functions

use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};

use crate::napi::bindings as napi;
use crate::raw::{Env, Local};

unsafe fn string(env: Env, s: impl AsRef<str>) -> Local {
    let s = s.as_ref();
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::create_string_utf8(
            env,
            s.as_bytes().as_ptr() as *const _,
            s.len(),
            result.as_mut_ptr(),
        ),
        napi::Status::Ok,
    );

    result.assume_init()
}

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
                string(env, "neon threadsafe function"),
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
    pub unsafe fn reference(&mut self, env: Env) -> Result<(), napi::Status> {
        let status = napi::ref_threadsafe_function(env, self.tsfn.0);

        if status == napi::Status::Ok {
            Ok(())
        } else {
            Err(status)
        }
    }

    /// Unreferences a threadsafe function to allow exiting the event loop before it has been dropped.
    /// Safety: `Env` must be valid for the current thread
    pub unsafe fn unref(&mut self, env: Env) -> Result<(), napi::Status> {
        let status = napi::unref_threadsafe_function(env, self.tsfn.0);

        if status == napi::Status::Ok {
            Ok(())
        } else {
            Err(status)
        }
    }

    // Provides a C ABI wrapper for a napi callback notifying us about tsfn
    // being finalized.
    unsafe extern "C" fn finalize(_env: Env, data: *mut c_void, _hint: *mut c_void) {
        let is_finalized = Arc::from_raw(data as *mut Mutex<bool>);

        *is_finalized.lock().unwrap() = true;
    }

    // Provides a C ABI wrapper for invoking the user supplied function pointer
    unsafe extern "C" fn callback(
        env: Env,
        _js_callback: napi::Value,
        _context: *mut c_void,
        data: *mut c_void,
    ) {
        let Callback { callback, data } = *Box::from_raw(data as *mut Callback<T>);

        // Event loop has terminated
        let env = if env.is_null() { None } else { Some(env) };

        callback(env, data);
    }
}

impl<T> Drop for ThreadsafeFunction<T> {
    fn drop(&mut self) {
        let is_finalized = self.is_finalized.lock().unwrap();

        // tsfn was already finalized by `Environment::CleanupHandles()` in
        // Node.js
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
