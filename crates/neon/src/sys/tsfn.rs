//! Idiomatic Rust wrappers for N-API threadsafe functions

use std::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr,
    sync::{Arc, Mutex},
};

use super::{bindings as napi, no_panic::FailureBoundary, raw::Env};

const BOUNDARY: FailureBoundary = FailureBoundary {
    both: "A panic and exception occurred while executing a `neon::event::Channel::send` callback",
    exception: "An exception occurred while executing a `neon::event::Channel::send` callback",
    panic: "A panic occurred while executing a `neon::event::Channel::send` callback",
};

#[cfg(feature = "napi-8")]
// Identifies state stored in the async resource of a threadsafe function
static TSFN_TAG: once_cell::sync::Lazy<crate::sys::TypeTag> = once_cell::sync::Lazy::new(|| {
    let mut tag = *crate::MODULE_TAG;
    tag.upper = crate::UpperTypeTag::Tsfn as u64;
    tag
});

#[derive(Debug)]
struct Tsfn(napi::ThreadsafeFunction);

unsafe impl Send for Tsfn {}

unsafe impl Sync for Tsfn {}

#[derive(Debug)]
/// Threadsafe Function encapsulate a Rust function pointer and N-API threadsafe
/// function for scheduling tasks to execute on a JavaScript thread.
pub struct ThreadsafeFunction<T> {
    tsfn: Tsfn,
    state: Arc<Mutex<State>>,
    callback: fn(Option<Env>, T),
}

#[derive(Debug)]
struct State {
    is_finalized: bool,
    has_ref: bool,
}

#[derive(Debug)]
struct Callback<T> {
    callback: fn(Option<Env>, T),
    data: T,
}

/// Error returned when scheduling a threadsafe function with some data
pub struct CallError;

unsafe extern "C" fn has_ref_callback(env: Env, info: napi::CallbackInfo) -> napi::Value {
    let create_bool = |result| {
        let mut out = MaybeUninit::uninit();
        assert_eq!(
            napi::get_boolean(env, result, out.as_mut_ptr()),
            napi::Status::Ok
        );
        out.assume_init()
    };

    // If we hit _any_ failure condition, assume the threadsafe function is referenced
    let bail = || create_bool(true);

    let this = {
        let mut this = MaybeUninit::uninit();

        if napi::get_cb_info(
            env,
            info,
            ptr::null_mut(),
            ptr::null_mut(),
            this.as_mut_ptr(),
            ptr::null_mut(),
        ) != napi::Status::Ok
        {
            return bail();
        }

        this.assume_init()
    };

    #[cfg(feature = "napi-8")]
    {
        let mut has_tag = false;
        let status =
            napi::check_object_type_tag(env, this, &*TSFN_TAG as *const _, &mut has_tag as *mut _);

        if status != napi::Status::Ok || !has_tag {
            return bail();
        }
    }

    let mut state = MaybeUninit::uninit();

    if napi::unwrap(env, this, state.as_mut_ptr()) != napi::Status::Ok {
        return bail();
    }

    let state = &*state.assume_init().cast::<Mutex<State>>();
    let is_ref = state.lock().map(|state| state.has_ref).unwrap_or(true);

    create_bool(is_ref)
}

unsafe extern "C" fn drop_state(_env: Env, data: *mut c_void, _hint: *mut c_void) {
    drop(Arc::<Mutex<State>>::from_raw(data.cast()))
}

unsafe fn create_async_resource(env: Env, state: Arc<Mutex<State>>) -> napi::Value {
    let has_ref_fn_name = "hasRef";

    let has_ref_fn = {
        let mut has_ref_fn = MaybeUninit::uninit();

        assert_eq!(
            napi::create_function(
                env,
                has_ref_fn_name.as_ptr().cast(),
                has_ref_fn_name.len(),
                Some(has_ref_callback),
                ptr::null_mut(),
                has_ref_fn.as_mut_ptr(),
            ),
            napi::Status::Ok,
        );

        has_ref_fn.assume_init()
    };

    let resource = {
        let mut resource = MaybeUninit::uninit();

        assert_eq!(
            napi::create_object(env, resource.as_mut_ptr()),
            napi::Status::Ok
        );

        resource.assume_init()
    };

    let has_ref_key = {
        let mut key = MaybeUninit::uninit();

        assert_eq!(
            napi::create_string_utf8(
                env,
                has_ref_fn_name.as_ptr().cast(),
                has_ref_fn_name.len(),
                key.as_mut_ptr()
            ),
            napi::Status::Ok
        );

        key.assume_init()
    };

    assert_eq!(
        napi::set_property(env, resource, has_ref_key, has_ref_fn),
        napi::Status::Ok
    );

    assert_eq!(
        napi::wrap(
            env,
            resource,
            Arc::into_raw(state) as *mut _,
            Some(drop_state),
            ptr::null_mut(),
            ptr::null_mut(),
        ),
        napi::Status::Ok
    );

    #[cfg(feature = "napi-8")]
    assert_eq!(
        napi::type_tag_object(env, resource, &*TSFN_TAG),
        napi::Status::Ok
    );

    resource
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
        let state = Arc::new(Mutex::new(State {
            is_finalized: false,
            has_ref: true,
        }));

        assert_eq!(
            napi::create_threadsafe_function(
                env,
                ptr::null_mut(),
                create_async_resource(env, state.clone()),
                super::string(env, "neon threadsafe function"),
                max_queue_size,
                // Always set the reference count to 1. Prefer using
                // Rust `Arc` to maintain the struct.
                1,
                Arc::into_raw(state.clone()) as *mut _,
                Some(Self::finalize),
                std::ptr::null_mut(),
                Some(Self::callback),
                result.as_mut_ptr(),
            ),
            napi::Status::Ok,
        );

        Self {
            tsfn: Tsfn(result.assume_init()),
            state,
            callback,
        }
    }

    /// Schedule a threadsafe function to be executed with some data
    pub fn call(
        &self,
        data: T,
        is_blocking: Option<napi::ThreadsafeFunctionCallMode>,
    ) -> Result<(), CallError> {
        let is_blocking = is_blocking.unwrap_or(napi::ThreadsafeFunctionCallMode::Blocking);

        let callback = Box::into_raw(Box::new(Callback {
            callback: self.callback,
            data,
        }));

        // Hold the lock before entering `call_threadsafe_function` so that
        // `finalize_cb` would never complete.
        let mut state = self.state.lock().unwrap();

        let status = {
            if state.is_finalized {
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
                state.is_finalized = true;
            }

            // If the call failed, the callback won't execute
            let _ = unsafe { Box::from_raw(callback) };

            Err(CallError)
        }
    }

    /// References a threadsafe function to prevent exiting the event loop until it has been dropped. (Default)
    /// Safety: `Env` must be valid for the current thread
    pub unsafe fn reference(&self, env: Env) {
        let mut state = self.state.lock().unwrap();
        assert_eq!(
            napi::ref_threadsafe_function(env, self.tsfn.0),
            napi::Status::Ok,
        );
        state.has_ref = true;
    }

    /// Unreferences a threadsafe function to allow exiting the event loop before it has been dropped.
    /// Safety: `Env` must be valid for the current thread
    pub unsafe fn unref(&self, env: Env) {
        let mut state = self.state.lock().unwrap();
        assert_eq!(
            napi::unref_threadsafe_function(env, self.tsfn.0),
            napi::Status::Ok,
        );
        state.has_ref = false;
    }

    // Provides a C ABI wrapper for a napi callback notifying us about tsfn
    // being finalized.
    unsafe extern "C" fn finalize(_env: Env, data: *mut c_void, _hint: *mut c_void) {
        let state = Arc::from_raw(data as *mut Mutex<State>);

        state.lock().unwrap().is_finalized = true;
    }

    // Provides a C ABI wrapper for invoking the user supplied function pointer
    // On panic or exception, creates a fatal exception of the form:
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

        BOUNDARY.catch_failure(env, None, move |env| {
            callback(env, data);
            ptr::null_mut()
        });
    }
}

impl<T> Drop for ThreadsafeFunction<T> {
    fn drop(&mut self) {
        let state = self.state.lock().unwrap();

        // tsfn was already finalized by `Environment::CleanupHandles()` in Node.js
        if state.is_finalized {
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
