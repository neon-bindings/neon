use raw::Local;

extern "system" {

    #[link_name = "NeonSys_Error_NewTypeError"]
    pub fn new_type_error(out: &mut Local, msg: Local) -> bool;

    #[link_name = "NeonSys_Error_CStringToTypeError"]
    pub fn cstring_to_type_error(out: &mut Local, msg: *const u8) -> bool;

    #[link_name = "NeonSys_Error_Throw"]
    pub fn throw(val: Local);

    #[link_name = "NeonSys_Error_ThrowTypeErrorFromCString"]
    pub fn throw_type_error_from_cstring(msg: *const u8);

}
