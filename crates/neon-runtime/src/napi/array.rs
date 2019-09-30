use raw::{Local, Isolate};

pub unsafe extern "C" fn new(_out: &mut Local, _isolate: *mut Isolate, _length: u32) { unimplemented!() }

pub unsafe extern "C" fn len(_array: Local) -> u32 { unimplemented!() }
