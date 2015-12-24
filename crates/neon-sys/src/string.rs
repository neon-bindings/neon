use raw::{Local, Isolate};

extern "system" {

    #[link_name = "NeonSys_String_New"]
    pub fn new(out: &mut Local, isolate: *mut Isolate, data: *const u8, len: i32) -> bool;

    #[link_name = "NeonSys_String_Utf8Length"]
    pub fn utf8_len(str: Local) -> isize;

    #[link_name = "NeonSys_String_Data"]
    pub fn data(out: *mut u8, len: isize, str: Local) -> isize;

}
