use raw::Local;

extern "system" {

    #[link_name = "NeonSys_Error_NewTypeError"]
    pub fn NewTypeError(out: &mut Local, msg: *const u8) -> bool;

    #[link_name = "NeonSys_Error_ThrowAny"]
    pub fn ThrowAny(val: Local);

    #[link_name = "NeonSys_Error_ThrowTypeError"]
    pub fn ThrowTypeError(msg: *const u8);

}
