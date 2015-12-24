use raw::{Local, Isolate};

extern "system" {

    #[link_name = "NeonSys_Array_New"]
    pub fn New(out: &mut Local, isolate: *mut Isolate, length: u32);

    #[link_name = "NeonSys_Array_Length"]
    pub fn Length(array: Local) -> u32;

}
