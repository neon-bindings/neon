use raw::{Isolate, Local};

extern "C" {

    #[link_name = "NeonSys_Object_New"]
    pub fn new(out: &mut Local);

    #[link_name = "NeonSys_Object_GetOwnPropertyNames"]
    pub fn get_own_property_names(out: &mut Local, object: Local) -> bool;

    #[link_name = "NeonSys_Object_GetIsolate"]
    pub fn get_isolate(obj: Local) -> *mut Isolate;

    #[link_name = "NeonSys_Object_Get_Index"]
    pub fn get_index(out: &mut Local, object: Local, index: u32) -> bool;

    #[link_name = "NeonSys_Object_Set_Index"]
    pub fn set_index(out: &mut bool, object: Local, index: u32, val: Local) -> bool;

    #[link_name = "NeonSys_Object_Get_String"]
    pub fn get_string(out: &mut Local, object: Local, key: *const u8, len: i32) -> bool;

    #[link_name = "NeonSys_Object_Set_String"]
    pub fn set_string(out: &mut bool, object: Local, key: *const u8, len: i32, val: Local) -> bool;

    #[link_name = "NeonSys_Object_Get"]
    pub fn get(out: &mut Local, object: Local, key: Local) -> bool;

    #[link_name = "NeonSys_Object_Set"]
    pub fn set(out: &mut bool, object: Local, key: Local, val: Local) -> bool;

}
