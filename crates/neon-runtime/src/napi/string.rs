use raw::{Local, Isolate};

pub extern "C" fn new(out: &mut Local, isolate: *mut Isolate, data: *const u8, len: i32) -> bool { unimplemented!() }

pub extern "C" fn utf8_len(str: Local) -> isize { unimplemented!() }

pub extern "C" fn data(out: *mut u8, len: isize, str: Local) -> isize { unimplemented!() }
