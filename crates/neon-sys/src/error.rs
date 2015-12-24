use raw::Local;

extern "system" {

    #[link_name = "NeonSys_Error_NewTypeError"]
    pub fn new_type_error(out: &mut Local, msg: *const u8) -> bool;

    #[link_name = "NeonSys_Error_Throw"]
    pub fn throw(val: Local);

    #[link_name = "NeonSys_Error_ThrowTypeError"]
    pub fn throw_type_error(msg: *const u8);

}
