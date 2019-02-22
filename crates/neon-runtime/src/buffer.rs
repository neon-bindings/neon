//! Facilities for working with `node::Buffer`s.

use raw::{Isolate, Persistent};
use std::os::raw::c_void;

extern "C" {

    /// Initializes the `out` argumnent with a newly created `v8::Buffer` using the safe constructor.
    #[link_name = "Neon_Buffer_New"]
    pub fn new(out: &Persistent, isolate: *mut Isolate, size: u32) -> bool;

    /// Initializes the `out` argumnent with a newly created `v8::Buffer` using the unsafe constructor.
    #[link_name = "Neon_Buffer_Uninitialized"]
    pub fn uninitialized(out: &Persistent, isolate: *mut Isolate, size: u32) -> bool;

    /// Mutates the `base_out` and `size_out` arguments to access the data of a `node::Buffer` object.
    #[link_name = "Neon_Buffer_Data"]
    pub fn data<'a, 'b>(base_out: &'a mut *mut c_void, size_out: &'a mut usize, obj: &Persistent);

}
