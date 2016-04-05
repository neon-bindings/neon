use raw::Local;
use cslice::CMutSlice;

// Suppress a spurious rustc warning about the use of CMutSlice.
#[allow(improper_ctypes)]
extern "system" {

    #[link_name = "NeonSys_Buffer_New"]
    pub fn new(out: &mut Local, size: u32) -> bool;

    #[link_name = "NeonSys_Buffer_Data"]
    pub fn data<'a, 'b>(out: &'a mut CMutSlice<'b, u8>, obj: Local);

}
