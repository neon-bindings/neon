use raw::Local;
use std::os::raw::c_void;

pub unsafe extern "C" fn new(_out: &mut Local, _isolate: *mut c_void, _size: u32) -> bool { unimplemented!() }

pub unsafe extern "C" fn data<'a, 'b>(_base_out: &'a mut *mut c_void, _obj: Local) -> usize { unimplemented!() }
