//! Facilities for working with `String`s.

use raw::{Local, Isolate};

extern "system" {

    /// Mutates the `out` argument provided to refer to a newly created `String`. Returns `false`
    /// if the value couldn't be created.
    #[link_name = "NeonSys_String_New"]
    pub fn new(out: &mut Local, isolate: *mut Isolate, data: *const u8, len: i32) -> bool;

    /// Gets the length of a `String`.
    #[link_name = "NeonSys_String_Utf8Length"]
    pub fn utf8_len(str: Local) -> isize;

    /// Writes data to a `String` and returns the number of bytes writen.
    #[link_name = "NeonSys_String_Data"]
    pub fn data(out: *mut u8, len: isize, str: Local) -> isize;

}
