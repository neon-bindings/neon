use raw::Local;

pub unsafe extern "C" fn throw(_val: Local) { unimplemented!() }

pub unsafe extern "C" fn new_error(_out: &mut Local, _msg: Local) { unimplemented!() }

pub unsafe extern "C" fn new_type_error(_out: &mut Local, _msg: Local) { unimplemented!() }

pub unsafe extern "C" fn new_range_error(_out: &mut Local, _msg: Local) { unimplemented!() }

pub unsafe extern "C" fn throw_error_from_utf8(_msg: *const u8, _len: i32) { unimplemented!() }
