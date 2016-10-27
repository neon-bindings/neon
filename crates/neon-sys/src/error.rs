use raw::Local;

extern "C" {

    #[link_name = "NeonSys_Error_Throw"]
    pub fn throw(val: Local);

    #[link_name = "NeonSys_Error_NewError"]
    pub fn new_error(out: &mut Local, msg: Local);

    #[link_name = "NeonSys_Error_NewTypeError"]
    pub fn new_type_error(out: &mut Local, msg: Local);

    #[link_name = "NeonSys_Error_NewReferenceError"]
    pub fn new_reference_error(out: &mut Local, msg: Local);

    #[link_name = "NeonSys_Error_NewRangeError"]
    pub fn new_range_error(out: &mut Local, msg: Local);

    #[link_name = "NeonSys_Error_NewSyntaxError"]
    pub fn new_syntax_error(out: &mut Local, msg: Local);

    #[link_name = "NeonSys_Error_ThrowErrorFromCString"]
    pub fn throw_error_from_cstring(msg: *const u8);

    #[link_name = "NeonSys_Error_ThrowTypeErrorFromCString"]
    pub fn throw_type_error_from_cstring(msg: *const u8);

    #[link_name = "NeonSys_Error_ThrowReferenceErrorFromCString"]
    pub fn throw_reference_error_from_cstring(msg: *const u8);

    #[link_name = "NeonSys_Error_ThrowRangeErrorFromCString"]
    pub fn throw_range_error_from_cstring(msg: *const u8);

    #[link_name = "NeonSys_Error_ThrowSyntaxErrorFromCString"]
    pub fn throw_syntax_error_from_cstring(msg: *const u8);

}
