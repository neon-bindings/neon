use crate::raw::Local;
use std::os::raw::c_void;

pub unsafe extern "C" fn schedule(_task: *mut c_void,
                                  _perform: unsafe extern fn(*mut c_void) -> *mut c_void,
                                  _complete: unsafe extern fn(*mut c_void, *mut c_void, &mut Local),
                                  _callback: Local) { unimplemented!() }
