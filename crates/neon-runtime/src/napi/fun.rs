use std::os::raw::c_void;
use call::CCallback;
use raw::Local;

pub extern "C" fn new(out: &mut Local, isolate: *mut c_void, callback: CCallback) -> bool { unimplemented!() }

pub extern "C" fn new_template(out: &mut Local, isolate: *mut c_void, callback: CCallback) -> bool { unimplemented!() }

pub extern "C" fn get_dynamic_callback(obj: Local) -> *mut c_void { unimplemented!() }

pub extern "C" fn call(out: &mut Local, isolate: *mut c_void, fun: Local, this: Local, argc: i32, argv: *mut c_void) -> bool { unimplemented!() }

pub extern "C" fn construct(out: &mut Local, isolate: *mut c_void, fun: Local, argc: i32, argv: *mut c_void) -> bool { unimplemented!() }
