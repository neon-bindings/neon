use raw::Local;
use std::os::raw::c_void;

pub unsafe extern "C" fn new(_out: &mut Local, _size: u32) -> bool { unimplemented!() }

pub unsafe extern "C" fn uninitialized(_out: &mut Local, _size: u32) -> bool { unimplemented!() }

pub unsafe extern "C" fn data<'a, 'b>(_base_out: &'a mut *mut c_void, _obj: Local) -> usize { unimplemented!() }
