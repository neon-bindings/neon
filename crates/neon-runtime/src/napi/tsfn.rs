use std::ffi::c_void;
use std::mem::MaybeUninit;

use nodejs_sys as napi;

use raw::{Local, Env};

pub use nodejs_sys::napi_threadsafe_function_call_mode as CallMode;
pub use nodejs_sys::napi_status as Status;

unsafe fn string(env: Env, s: impl AsRef<str>) -> Local {
    let s = s.as_ref();
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_create_string_utf8(
            env,
            s.as_bytes().as_ptr() as *const _,
            s.len(),
            result.as_mut_ptr(),
        ),
        napi::napi_status::napi_ok,
    );

    result.assume_init()
}

#[derive(Debug)]
struct Tsfn(napi::napi_threadsafe_function);

unsafe impl Send for Tsfn {}
unsafe impl Sync for Tsfn {}

#[derive(Debug)]
pub struct ThreadsafeFunction<T> {
    tsfn: Tsfn,
    callback: fn(Env, T),
}

#[derive(Debug)]
struct Callback<T> {
    callback: fn(Env, T),
    data: T,
}

pub struct CallError<T> {
    kind: Status,
    data: T,
}

impl<T> CallError<T> {
    pub fn kind(&self) -> Status {
        self.kind
    }

    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T: Send + 'static> ThreadsafeFunction<T> {
    // Caller must maintain that `Env` is valid for the current thread
    pub unsafe fn new(
        env: Env,
        callback: fn(Env, T),        
    ) -> Self {
        Self::with_capacity(env, 0, callback)
    }

    pub unsafe fn with_capacity(
        env: Env,
        max_queue_size: usize,
        callback: fn(Env, T),
    ) -> Self {
        let mut result = MaybeUninit::uninit();

        assert_eq!(
            napi::napi_create_threadsafe_function(
                env,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                string(env, "neon threadsafe function"),
                max_queue_size,
                // Always set the reference count to 1. Prefer using
                // Rust `Arc` to maintain the struct.
                1,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                Some(Self::callback),
                result.as_mut_ptr(),
            ),
            napi::napi_status::napi_ok,
        );
    
        Self {
            tsfn: Tsfn(result.assume_init()),
            callback,
        }
    }

    pub fn call(&self, data: T, is_blocking: CallMode) -> Result<(), CallError<T>> {
        let callback = Box::into_raw(Box::new(Callback {
            callback: self.callback,
            data,
        }));

        let status = unsafe {
            napi::napi_call_threadsafe_function(
                self.tsfn.0,
                callback as *mut _,
                is_blocking,
            )
        };

        if status == Status::napi_ok {
            Ok(())
        } else {
            // If the call failed, the callback won't execute
            let callback = unsafe { Box::from_raw(callback) };

            Err(CallError {
                kind: status,
                data: callback.data,
            })
        }
    }

    pub unsafe fn reference(&mut self, env: Env) {
        assert_eq!(
            napi::napi_ref_threadsafe_function(
                env,
                self.tsfn.0,
            ),
            napi::napi_status::napi_ok,
        );
    }

    pub unsafe fn unref(&mut self, env: Env) {
        assert_eq!(
            napi::napi_unref_threadsafe_function(
                env,
                self.tsfn.0,
            ),
            napi::napi_status::napi_ok,
        );
    }

    unsafe extern "C" fn callback(
        env: Env,
        _js_callback: napi::napi_value,
        _context: *mut c_void,
        data: *mut c_void,
    ) {
        // Event loop may be terminated
        if data.is_null() {
            return;
        }

        let Callback {
            callback,
            data,
        } = *Box::from_raw(data as *mut Callback<T>);

        // Event loop has terminated
        if env.is_null() {
            eprintln!("This is surprising");
            return;
        }

        callback(env, data);
    }
}

impl<T> Drop for ThreadsafeFunction<T> {
    fn drop(&mut self) {
        unsafe {
            napi::napi_release_threadsafe_function(
                self.tsfn.0,
                napi::napi_threadsafe_function_release_mode::napi_tsfn_release,
            );
        };
    }
}
