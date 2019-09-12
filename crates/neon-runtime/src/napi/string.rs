//! Facilities for working with `v8::String`s.

use raw::{Local, Isolate};

// FIXME(napi): #[link_name = "Neon_String_New"]
pub extern "C" fn new(out: &mut Local, isolate: *mut Isolate, data: *const u8, len: i32) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_String_Utf8Length"]
pub extern "C" fn utf8_len(str: Local) -> isize { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_String_Data"]
pub extern "C" fn data(out: *mut u8, len: isize, str: Local) -> isize { unimplemented!() }
