use raw::{Local, Isolate};

extern "system" {

    #[link_name = "NeonSys_Array_New"]
    pub fn new(out: &mut Local, isolate: *mut Isolate, length: u32);

    #[link_name = "NeonSys_Array_Length"]
    pub fn len(array: Local) -> u32;

}
