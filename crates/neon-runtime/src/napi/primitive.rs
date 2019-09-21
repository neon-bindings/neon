use raw::{Local, Isolate};

pub unsafe extern "C" fn undefined(_out: &mut Local) { unimplemented!() }

pub unsafe extern "C" fn null(_out: &mut Local) { unimplemented!() }

pub unsafe extern "C" fn boolean(_out: &mut Local, _b: bool) { unimplemented!() }

pub unsafe extern "C" fn boolean_value(_p: Local) -> bool { unimplemented!() }

// DEPRECATE(0.2)
pub unsafe extern "C" fn integer(_out: &mut Local, _isolate: *mut Isolate, _x: i32) { unimplemented!() }

pub unsafe extern "C" fn is_u32(_p: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_i32(_p: Local) -> bool { unimplemented!() }

// DEPRECATE(0.2)
pub unsafe extern "C" fn integer_value(_p: Local) -> i64 { unimplemented!() }

pub unsafe extern "C" fn number(_out: &mut Local, _isolate: *mut Isolate, _v: f64) { unimplemented!() }

pub unsafe extern "C" fn number_value(_p: Local) -> f64 { unimplemented!() }
