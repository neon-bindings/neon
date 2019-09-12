//! Facilities for working with `v8::Array`s.

use raw::{Local, Isolate};

// FIXME(napi): #[link_name = "Neon_Array_New"]
pub extern "C" fn new(out: &mut Local, isolate: *mut Isolate, length: u32) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Array_Length"]
pub extern "C" fn len(array: Local) -> u32 { unimplemented!() }
