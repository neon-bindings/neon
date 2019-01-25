//! Facilities for running background tasks in the libuv thread pool.

use raw::{Local, Persistent};
use std::os::raw::c_void;

extern "C" {

    /// Schedules a background task.
    #[link_name = "Neon_Task_Schedule"]
    pub fn schedule(task: *mut c_void,
                    perform: unsafe extern fn(*mut c_void) -> *mut c_void,
                    complete: unsafe extern fn(*mut c_void, *mut c_void, &mut Local),
                    callback: &Persistent);

}
