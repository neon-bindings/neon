//! Facilities for working with `v8::ArrayBuffer`s.

use raw::Local;
use cslice::CMutSlice;
use std::os::raw::c_void;

// Suppress a spurious rustc warning about the use of CMutSlice.
#[allow(improper_ctypes)]
extern "C" {

    /// Mutates the `out` argument provided to refer to a newly created `v8::ArrayBuffer` object.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "Neon_ArrayBuffer_New"]
    pub fn new(out: &mut Local, isolate: *mut c_void, size: u32) -> bool;

    /// Mutates the `out` argument provided populating the `data` and `len` properties.
    #[link_name = "Neon_ArrayBuffer_Data"]
    pub fn data<'a, 'b>(out: &'a mut CMutSlice<'b, u8>, obj: Local);

}
