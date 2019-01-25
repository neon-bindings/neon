//! Facilities for working with `v8::ArrayBuffer`s.

use raw::{Isolate, Local, Persistent};
use std::os::raw::c_void;

// FIXME: rename init_xxx functions back to new_xxx

extern "C" {

    /// Initializes the `out` argument provided to refer to a newly created `v8::ArrayBuffer` object.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "Neon_ArrayBuffer_Init"]
    pub fn init(out: &Persistent, isolate: *mut Isolate, size: u32) -> bool;

    /// Mutates the `out` argument provided to refer to a newly created `v8::ArrayBuffer` object.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "Neon_ArrayBuffer_New"]
    pub fn new(out: &mut Local, isolate: *mut c_void, size: u32) -> bool;

    /// Mutates the `base_out` and `size_out` arguments to access the data of a `v8::ArrayBuffer` object.
    #[link_name = "Neon_ArrayBuffer_Data"]
    pub fn data<'a, 'b>(base_out: &'a mut *mut c_void, size_out: &'a mut usize, obj: &Persistent);
}
