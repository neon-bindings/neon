use raw::Local;
use buf::Buf;

extern "system" {

    #[link_name = "NeonSys_Buffer_New"]
    pub fn new(out: &mut Local, size: u32) -> bool;

    #[link_name = "NeonSys_Buffer_Data"]
    pub fn data<'a, 'b>(out: &'a mut Buf<'b>, obj: Local);

}
