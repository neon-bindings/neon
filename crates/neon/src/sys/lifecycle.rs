//! # Environment life cycle APIs
//!
//! These APIs map to the life cycle of a specific "Agent" or self-contained
//! environment. If a Neon module is loaded multiple times (Web Workers, worker
//! threads), these API will be handle data associated with a specific instance.
//!
//! See the [N-API Lifecycle][npai-docs] documentation for more details.
//!
//! [napi-docs]: https://nodejs.org/api/n-api.html#n_api_environment_life_cycle_apis

use std::{mem::MaybeUninit, os::raw::c_void, ptr};

use super::{bindings as napi, raw::Env};

/// # Safety
/// `env` must point to a valid `napi_env` for this thread
pub unsafe fn set_instance_data<T: Send + 'static>(env: Env, data: T) -> *mut T {
    let data = Box::into_raw(Box::new(data));

    assert_eq!(
        napi::set_instance_data(env, data.cast(), Some(drop_box::<T>), ptr::null_mut(),),
        napi::Status::Ok,
    );

    data
}

/// # Safety
/// * `T` must be the same type used in `set_instance_data`
/// * Caller must ensure reference does not outlive `Env`
/// * Return value may be `null`
/// * `env` must point to a valid `napi_env` for this thread
pub unsafe fn get_instance_data<T: Send + 'static>(env: Env) -> *mut T {
    let mut data = MaybeUninit::uninit();

    assert_eq!(
        napi::get_instance_data(env, data.as_mut_ptr(),),
        napi::Status::Ok,
    );

    data.assume_init().cast()
}

unsafe extern "C" fn drop_box<T>(_env: Env, data: *mut c_void, _hint: *mut c_void) {
    drop(Box::<T>::from_raw(data.cast()));
}
