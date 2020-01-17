use raw::{Local, Env};

pub unsafe extern "C" fn new(_out: &mut Local, _env: Env, _length: u32) { unimplemented!() }

pub unsafe extern "C" fn len(_array: Local) -> u32 { unimplemented!() }
