//! Facilities for working with `node::Buffer`s.

use raw::Local;
use std::os::raw::c_void;

extern "C" {

    /// Mutates the `out` argument provided to refer to a newly created and zero-filled `node::Buffer` object.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "Neon_Buffer_New"]
    pub fn new(out: &mut Local, size: u32) -> bool;

    /// Mutates the `out` argument provided to refer to a newly created `node::Buffer` object.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "Neon_Buffer_Uninitialized"]
    pub fn uninitialized(out: &mut Local, size: u32) -> bool;

    /// Mutates the `base_out` and `size_out` arguments to access the data of a `node::Buffer` object.
    #[link_name = "Neon_Buffer_Data"]
    pub fn data<'a, 'b>(base_out: &'a mut *mut c_void, obj: Local) -> usize;

}
