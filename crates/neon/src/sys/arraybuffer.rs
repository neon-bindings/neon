#[cfg(feature = "external-buffers")]
use std::os::raw::c_void;
use std::{mem::MaybeUninit, ptr::null_mut, slice};

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn new(env: Env, len: usize) -> Result<Local, napi::Status> {
    let mut buf = MaybeUninit::uninit();
    let status = unsafe { napi::create_arraybuffer(env, len, null_mut(), buf.as_mut_ptr()) };

    match status {
        Err(err @ napi::Status::PendingException) => return Err(err),
        status => status.unwrap(),
    };

    Ok(unsafe { buf.assume_init() })
}

#[cfg(feature = "external-buffers")]
pub unsafe fn new_external<T>(env: Env, data: T) -> Local
where
    T: AsMut<[u8]> + Send,
{
    // Safety: Boxing could move the data; must box before grabbing a raw pointer
    let mut data = Box::new(data);
    let buf = data.as_mut().as_mut();
    let length = buf.len();
    let mut result = MaybeUninit::uninit();

    unsafe {
        napi::create_external_arraybuffer(
            env,
            buf.as_mut_ptr() as *mut _,
            length,
            Some(drop_external::<T>),
            Box::into_raw(data) as *mut _,
            result.as_mut_ptr(),
        )
        .unwrap();

        result.assume_init()
    }
}

#[cfg(feature = "external-buffers")]
unsafe extern "C" fn drop_external<T>(_env: Env, _data: *mut c_void, hint: *mut c_void) {
    drop(unsafe { Box::<T>::from_raw(hint as *mut _) });
}

/// # Safety
/// * Caller must ensure `env` and `buf` are valid
/// * The lifetime `'a` does not exceed the lifetime of `Env` or `buf`
pub unsafe fn as_mut_slice<'a>(env: Env, buf: Local) -> &'a mut [u8] {
    let mut data = MaybeUninit::uninit();
    let mut size = 0usize;

    unsafe {
        napi::get_arraybuffer_info(env, buf, data.as_mut_ptr(), &mut size as *mut _).unwrap();
    }

    if size == 0 {
        return &mut [];
    }

    unsafe { slice::from_raw_parts_mut(data.assume_init().cast(), size) }
}

/// # Safety
/// * Caller must ensure `env` and `buf` are valid
pub unsafe fn size(env: Env, buf: Local) -> usize {
    let mut data = MaybeUninit::uninit();
    let mut size = 0usize;

    unsafe {
        napi::get_arraybuffer_info(env, buf, data.as_mut_ptr(), &mut size as *mut _).unwrap();
    }

    size
}
