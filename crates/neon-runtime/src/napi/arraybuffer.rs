use crate::raw::{Env, Local};
use std::os::raw::c_void;
use std::ptr::null_mut;

use crate::napi::bindings as napi;

pub unsafe extern "C" fn new(out: &mut Local, env: Env, size: u32) -> bool {
    let status = napi::create_arraybuffer(env, size as usize, null_mut(), out as *mut _);

    status == napi::Status::Ok
}

pub unsafe extern "C" fn data<'a, 'b>(env: Env, base_out: &'a mut *mut c_void, obj: Local) -> usize {
    let mut size = 0;
    assert_eq!(
        napi::get_arraybuffer_info(env, obj, base_out as *mut _, &mut size as *mut _),
        napi::Status::Ok,
    );
    size
}
