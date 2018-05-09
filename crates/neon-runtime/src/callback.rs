use std::os::raw::c_void;
use std::os::raw::c_int;
use raw::Local;

extern "C" {
    #[link_name = "Neon_Callback_New"]
    pub fn new(value: Local) -> *mut c_void;

    #[link_name = "Neon_Callback_Call"]
    pub fn call(callback: *mut c_void, argc: c_int, argv: &mut [Local]);
}
