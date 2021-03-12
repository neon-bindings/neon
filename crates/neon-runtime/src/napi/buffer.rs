use crate::raw::{Env, Local};
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::ptr::null_mut;

use crate::napi::bindings as napi;

pub unsafe extern "C" fn new(env: Env, out: &mut Local, size: u32) -> bool {
    let mut bytes = null_mut();
    let status = napi::create_buffer(env, size as usize, &mut bytes as *mut _, out as *mut _);
    if status == napi::Status::Ok {
        // zero-initialize it. If performance is critical, JsBuffer::uninitialized can be used
        // instead.
        std::ptr::write_bytes(bytes, 0, size as usize);
        true
    } else {
        false
    }
}

pub unsafe extern "C" fn uninitialized(env: Env, out: &mut Local, size: u32) -> bool {
    let mut bytes = null_mut();
    let status = napi::create_buffer(env, size as usize, &mut bytes as *mut _, out as *mut _);
    status == napi::Status::Ok
}

pub unsafe fn new_external<T>(env: Env, data: T) -> Local
where
    T: AsMut<[u8]> + Send,
{
    // Safety: Boxing could move the data; must box before grabbing a raw pointer
    let mut data = Box::new(data);
    let buf = data.as_mut().as_mut();
    let length = buf.len();
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::create_external_buffer(
            env,
            length,
            buf.as_mut_ptr() as *mut _,
            Some(drop_external::<T>),
            Box::into_raw(data) as *mut _,
            result.as_mut_ptr(),
        ),
        napi::Status::Ok,
    );

    result.assume_init()
}

pub unsafe extern "C" fn data<'a, 'b>(env: Env, base_out: &'a mut *mut c_void, obj: Local) -> usize {
    let mut size = 0;
    assert_eq!(
        napi::get_buffer_info(env, obj, base_out as *mut _, &mut size as *mut _),
        napi::Status::Ok,
    );
    size
}

unsafe extern "C" fn drop_external<T>(_env: Env, _data: *mut c_void, hint: *mut c_void) {
    Box::<T>::from_raw(hint as *mut _);
}
