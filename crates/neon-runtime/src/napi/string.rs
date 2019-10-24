use raw::{Local, Env};

pub unsafe extern "C" fn new(_out: &mut Local, _isolate: Env, _data: *const u8, _len: i32) -> bool { unimplemented!() }

pub unsafe extern "C" fn utf8_len(_str: Local) -> isize { unimplemented!() }

pub unsafe extern "C" fn data(_out: *mut u8, _len: isize, _str: Local) -> isize { unimplemented!() }
