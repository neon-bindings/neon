//! Facilities for working with `Arrays`s.

use raw::{Local, Isolate};

extern "system" {

    /// Mutates the `out` argument provided to refer to a newly created `Array`.
    #[link_name = "NeonSys_Array_New"]
    pub fn new(out: &mut Local, isolate: *mut Isolate, length: u32);

    /// Gets the length of an `Array`.
    #[link_name = "NeonSys_Array_Length"]
    pub fn len(array: Local) -> u32;

}
