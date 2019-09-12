//! Facilities for working with `node::Buffer`s.

use raw::Local;
use std::os::raw::c_void;

// FIXME(napi): #[link_name = "Neon_Buffer_New"]
pub extern "C" fn new(out: &mut Local, size: u32) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Buffer_Uninitialized"]
pub extern "C" fn uninitialized(out: &mut Local, size: u32) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Buffer_Data"]
pub extern "C" fn data<'a, 'b>(base_out: &'a mut *mut c_void, obj: Local) -> usize { unimplemented!() }
