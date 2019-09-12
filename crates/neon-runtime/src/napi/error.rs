//! Facilities for creating and throwing JS errors.

use raw::Local;

// FIXME(napi): #[link_name = "Neon_Error_Throw"]
pub extern "C" fn throw(val: Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Error_NewError"]
pub extern "C" fn new_error(out: &mut Local, msg: Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Error_NewTypeError"]
pub extern "C" fn new_type_error(out: &mut Local, msg: Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Error_NewRangeError"]
pub extern "C" fn new_range_error(out: &mut Local, msg: Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Error_ThrowErrorFromUtf8"]
pub extern "C" fn throw_error_from_utf8(msg: *const u8, len: i32) { unimplemented!() }
