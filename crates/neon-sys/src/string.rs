//! Facilities for working with `v8::String`s.

use raw::{Local, Isolate};

extern "C" {

    /// Mutates the `out` argument provided to refer to a newly created `v8::String`. Returns
    /// `false` if the value couldn't be created.
    #[link_name = "NeonSys_String_New"]
    pub fn new(out: &mut Local, isolate: *mut Isolate, data: *const u8, len: i32) -> bool;

    /// Gets the length of a `v8::String`.
    #[link_name = "NeonSys_String_Utf8Length"]
    pub fn utf8_len(str: Local) -> isize;

    /// Writes data to a `v8::String` and returns the number of bytes writen.
    #[link_name = "NeonSys_String_Data"]
    pub fn data(out: *mut u8, len: isize, str: Local) -> isize;

    #[link_name = "NeonSys_String_NewFromUCS2"]
    pub fn new_from_ucs2(out: &mut Local, isolate: *mut Isolate, data: *const u16, len: i32) -> bool;

    #[link_name = "NeonSys_String_UCS2Length"]
    pub fn ucs2_len(str: Local) -> isize;

    #[link_name = "NeonSys_String_UCS2Data"]
    pub fn ucs2_data(out: *mut u16, len: isize, str: Local) -> isize;

}
