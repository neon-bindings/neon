use crate::raw::{Env, Local};
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

pub unsafe extern "C" fn uninitialized(_out: &mut Local, _size: u32) -> bool { unimplemented!() }

pub unsafe extern "C" fn data<'a, 'b>(env: Env, base_out: &'a mut *mut c_void, obj: Local) -> usize {
    let mut size = 0;
    assert_eq!(
        napi::get_buffer_info(env, obj, base_out as *mut _, &mut size as *mut _),
        napi::Status::Ok,
    );
    size
}
