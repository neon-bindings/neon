use raw::{Local, Isolate};

extern "system" {

    #[link_name = "NeonSys_String_New"]
    pub fn new(out: &mut Local, isolate: *mut Isolate, data: *const u8, len: i32) -> bool;

    #[link_name = "NeonSys_String_Utf8Length"]
    pub fn utf8_len(str: Local) -> isize;

    #[link_name = "NeonSys_String_Data"]
    pub fn data(out: *mut u8, len: isize, str: Local) -> isize;

    #[link_name = "NeonSys_String_NewFromUCS2"]
    pub fn new_from_ucs2(out: &mut Local, isolate: *mut Isolate, data: *const u16, len: i32) -> bool;

    #[link_name = "NeonSys_String_UCS2Length"]
    pub fn ucs2_len(str: Local) -> isize;

    #[link_name = "NeonSys_String_UCS2Data"]
    pub fn ucs2_data(out: *mut u16, len: isize, str: Local) -> isize;

}
