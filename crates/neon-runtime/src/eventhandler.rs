//! Facilities for running a callback in the libuv main thread.

use raw::Local;
use std::os::raw::c_void;

extern "C" {

    /// Creates a new thread safe callback which can be used to execute a callback in the libuv main thread
    #[link_name = "Neon_EventHandler_New"]
    pub fn new(this: Local, callback: Local) -> *mut c_void;

    /// Executes the given callback in the libuv main thread
    #[link_name = "Neon_EventHandler_Call"]
    pub fn call(event_handler: *mut c_void, rust_callback: *mut c_void,
                complete: unsafe extern fn(Local, Local, *mut c_void));

    // Free the thread safe callback and any memory hold
    #[link_name = "Neon_EventHandler_Delete"]
    pub fn delete(event_handler: *mut c_void);

}
