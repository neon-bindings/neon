//! Facilities for working with `v8::Array`s.

use raw::{Local, Isolate};

extern "system" {

    /// Mutates the `out` argument provided to refer to a newly created `v8::Array`.
    #[link_name = "NeonSys_Array_New"]
    pub fn new(out: &mut Local, isolate: *mut Isolate, length: u32);

    /// Gets the length of an `v8::Array`.
    #[link_name = "NeonSys_Array_Length"]
    pub fn len(array: Local) -> u32;

}
