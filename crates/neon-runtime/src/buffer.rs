//! Facilities for working with `node::Buffer`s.

use raw::{Isolate, Local, Persistent};
use std::os::raw::c_void;

// FIXME: rename init_xxx functions back to new_xxx

extern "C" {

    /// Mutates the `out` argument provided to refer to a newly created and zero-filled `node::Buffer` object.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "Neon_Buffer_New"]
    pub fn new(out: &mut Local, size: u32) -> bool;

    /// Initializes the `out` argumnent with a newly created `v8::Buffer` using the safe constructor.
    #[link_name = "Neon_Buffer_Init_Safe"]
    pub fn init_safe(out: &Persistent, isolate: *mut Isolate, size: u32) -> bool;

    /// Initializes the `out` argumnent with a newly created `v8::Buffer` using the unsafe constructor.
    #[link_name = "Neon_Buffer_Init_Unsafe"]
    pub fn init_unsafe(out: &Persistent, isolate: *mut Isolate, size: u32) -> bool;

    /// Mutates the `out` argument provided to refer to a newly created `node::Buffer` object.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "Neon_Buffer_Uninitialized"]
    pub fn uninitialized(out: &mut Local, size: u32) -> bool;

    /// Mutates the `base_out` and `size_out` arguments to access the data of a `node::Buffer` object.
    #[link_name = "Neon_Buffer_Data"]
    pub fn data<'a, 'b>(base_out: &'a mut *mut c_void, size_out: &'a mut usize, obj: &Persistent);

}
