//! Facilities for working with `v8::String`s.

use raw::{Isolate, Persistent};

extern "C" {

    /// Mutates the `out` argument provided to refer to a newly created `v8::String`. Returns
    /// `false` if the value couldn't be created.
    #[link_name = "Neon_String_New"]
    pub fn new(out: &Persistent, isolate: *mut Isolate, data: *const u8, len: i32) -> bool;

    /// Gets the length of a `v8::String`.
    #[link_name = "Neon_String_Utf8Length"]
    pub fn utf8_len(str: &Persistent) -> isize;

    /// Writes data to a `v8::String` and returns the number of bytes writen.
    #[link_name = "Neon_String_Data"]
    pub fn data(out: *mut u8, len: isize, str: &Persistent) -> isize;

    /// Mutates the `out` argument to the result of applying the standard JS string conversion
    /// operation on the `from` argument.
    #[link_name = "Neon_String_ToString"]
    pub fn to_string(out: &Persistent, isolate: *mut Isolate, from: &Persistent) -> bool;

}
