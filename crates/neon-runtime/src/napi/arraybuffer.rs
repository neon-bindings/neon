//! Facilities for working with `v8::ArrayBuffer`s.

use raw::Local;
use std::os::raw::c_void;

// FIXME(napi): #[link_name = "Neon_ArrayBuffer_New"]
pub extern "C" fn new(out: &mut Local, isolate: *mut c_void, size: u32) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_ArrayBuffer_Data"]
pub extern "C" fn data<'a, 'b>(base_out: &'a mut *mut c_void, obj: Local) -> usize { unimplemented!() }
