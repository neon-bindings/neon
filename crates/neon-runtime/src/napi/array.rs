use raw::{Local, Isolate};

pub extern "C" fn new(out: &mut Local, isolate: *mut Isolate, length: u32) { unimplemented!() }

pub extern "C" fn len(array: Local) -> u32 { unimplemented!() }
