//! Facilities for working with `node::Buffer`s.

use raw::Local;
use cslice::CMutSlice;

// Suppress a spurious rustc warning about the use of CMutSlice.
#[allow(improper_ctypes)]
extern "system" {

    /// Mutates the `out` argument provided to refer to a newly created `node::Buffer` object.
    /// Returns `false` if the value couldn't be created.
    #[link_name = "NeonSys_Buffer_New"]
    pub fn new(out: &mut Local, size: u32) -> bool;

    /// Mutates the `out` argument provided populating the `data` and `len` properties.
    #[link_name = "NeonSys_Buffer_Data"]
    pub fn data<'a, 'b>(out: &'a mut CMutSlice<'b, u8>, obj: Local);

}
