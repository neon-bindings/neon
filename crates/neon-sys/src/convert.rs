use raw::Local;

extern "system" {

    #[link_name = "NeonSys_Convert_ToObject"]
    pub fn to_object(out: &mut Local, value: &Local) -> bool;

    #[link_name = "NeonSys_Convert_ToString"]
    pub fn to_string(out: &mut Local, value: Local) -> bool;

}
