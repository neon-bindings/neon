use raw::Local;
use std::os::raw::c_void;

pub extern "C" fn schedule(task: *mut c_void,
                           perform: unsafe extern fn(*mut c_void) -> *mut c_void,
                           complete: unsafe extern fn(*mut c_void, *mut c_void, &mut Local),
                           callback: Local) { unimplemented!() }
