use raw::Local;
use std::os::raw::c_void;

pub extern "C" fn new(out: &mut Local, size: u32) -> bool { unimplemented!() }

pub extern "C" fn uninitialized(out: &mut Local, size: u32) -> bool { unimplemented!() }

pub extern "C" fn data<'a, 'b>(base_out: &'a mut *mut c_void, obj: Local) -> usize { unimplemented!() }
