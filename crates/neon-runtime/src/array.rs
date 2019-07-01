//! Facilities for working with `v8::Array`s.

use raw::{Isolate, Local};

extern "C" {

    /// Mutates the `out` argument provided to refer to a newly created `v8::Array`.
    #[link_name = "Neon_Array_New"]
    pub fn new(out: &mut Local, isolate: *mut Isolate, length: u32);

    /// Gets the length of an `v8::Array`.
    #[link_name = "Neon_Array_Length"]
    pub fn len(array: Local) -> u32;

}
