//! Facilities for running background tasks in the libuv thread pool.

use raw::Local;
use std::os::raw::c_void;

// FIXME(napi): #[link_name = "Neon_Task_Schedule"]
pub extern "C" fn schedule(task: *mut c_void,
                           perform: unsafe extern fn(*mut c_void) -> *mut c_void,
                           complete: unsafe extern fn(*mut c_void, *mut c_void, &mut Local),
                           callback: Local) { unimplemented!() }
