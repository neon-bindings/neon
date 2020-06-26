use raw::{Env, Local};
use std::os::raw::c_void;
use std::ptr::null_mut;

use nodejs_sys as napi;

pub unsafe extern "C" fn new(out: &mut Local, env: Env, size: u32) -> bool {
    let status = napi::napi_create_arraybuffer(env, size as usize, null_mut(), out as *mut _);

    status == napi::napi_status::napi_ok
}

pub unsafe extern "C" fn data<'a, 'b>(_base_out: &'a mut *mut c_void, _obj: Local) -> usize {
    unimplemented!()
}
