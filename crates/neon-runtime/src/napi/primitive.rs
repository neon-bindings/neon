//! Facilities for working with primitive values.

use raw::{Local, Isolate};

// FIXME(napi): #[link_name = "Neon_Primitive_Undefined"]
pub extern "C" fn undefined(out: &mut Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Primitive_Null"]
pub extern "C" fn null(out: &mut Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Primitive_Boolean"]
pub extern "C" fn boolean(out: &mut Local, b: bool) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Primitive_BooleanValue"]
pub extern "C" fn boolean_value(p: Local) -> bool { unimplemented!() }

// DEPRECATE(0.2)
// FIXME(napi): #[link_name = "Neon_Primitive_Integer"]
pub extern "C" fn integer(out: &mut Local, isolate: *mut Isolate, x: i32) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Primitive_IsUint32"]
pub extern "C" fn is_u32(p: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Primitive_IsInt32"]
pub extern "C" fn is_i32(p: Local) -> bool { unimplemented!() }

// DEPRECATE(0.2)
// FIXME(napi): #[link_name = "Neon_Primitive_IntegerValue"]
pub extern "C" fn integer_value(p: Local) -> i64 { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Primitive_Number"]
pub extern "C" fn number(out: &mut Local, isolate: *mut Isolate, v: f64) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Primitive_NumberValue"]
pub extern "C" fn number_value(p: Local) -> f64 { unimplemented!() }
