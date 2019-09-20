use raw::{Isolate, Local};

pub extern "C" fn new(out: &mut Local) { unimplemented!() }

pub extern "C" fn get_own_property_names(out: &mut Local, object: Local) -> bool { unimplemented!() }

pub extern "C" fn get_isolate(obj: Local) -> *mut Isolate { unimplemented!() }

pub extern "C" fn get_index(out: &mut Local, object: Local, index: u32) -> bool { unimplemented!() }

pub extern "C" fn set_index(out: &mut bool, object: Local, index: u32, val: Local) -> bool { unimplemented!() }

pub extern "C" fn get_string(out: &mut Local, object: Local, key: *const u8, len: i32) -> bool { unimplemented!() }

pub extern "C" fn set_string(out: &mut bool, object: Local, key: *const u8, len: i32, val: Local) -> bool { unimplemented!() }

pub extern "C" fn get(out: &mut Local, object: Local, key: Local) -> bool { unimplemented!() }

pub extern "C" fn set(out: &mut bool, object: Local, key: Local, val: Local) -> bool { unimplemented!() }
