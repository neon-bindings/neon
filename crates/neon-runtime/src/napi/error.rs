use raw::Local;

pub extern "C" fn throw(val: Local) { unimplemented!() }

pub extern "C" fn new_error(out: &mut Local, msg: Local) { unimplemented!() }

pub extern "C" fn new_type_error(out: &mut Local, msg: Local) { unimplemented!() }

pub extern "C" fn new_range_error(out: &mut Local, msg: Local) { unimplemented!() }

pub extern "C" fn throw_error_from_utf8(msg: *const u8, len: i32) { unimplemented!() }
