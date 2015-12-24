use raw::Local;

extern "system" {

    #[link_name = "NeonSys_Convert_ToObject"]
    pub fn ToObject(out: &mut Local, value: &Local) -> bool;

    #[link_name = "NeonSys_Convert_ToString"]
    pub fn ToString(out: &mut Local, value: Local) -> bool;

}
