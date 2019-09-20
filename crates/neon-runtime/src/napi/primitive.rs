use raw::{Local, Isolate};

pub extern "C" fn undefined(out: &mut Local) { unimplemented!() }

pub extern "C" fn null(out: &mut Local) { unimplemented!() }

pub extern "C" fn boolean(out: &mut Local, b: bool) { unimplemented!() }

pub extern "C" fn boolean_value(p: Local) -> bool { unimplemented!() }

// DEPRECATE(0.2)
pub extern "C" fn integer(out: &mut Local, isolate: *mut Isolate, x: i32) { unimplemented!() }

pub extern "C" fn is_u32(p: Local) -> bool { unimplemented!() }

pub extern "C" fn is_i32(p: Local) -> bool { unimplemented!() }

// DEPRECATE(0.2)
pub extern "C" fn integer_value(p: Local) -> i64 { unimplemented!() }

pub extern "C" fn number(out: &mut Local, isolate: *mut Isolate, v: f64) { unimplemented!() }

pub extern "C" fn number_value(p: Local) -> f64 { unimplemented!() }
