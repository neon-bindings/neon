use raw::{Isolate, Local};

extern "system" {

    #[link_name = "NeonSys_Object_New"]
    pub fn New(out: &mut Local);

    #[link_name = "NeonSys_Object_GetOwnPropertyNames"]
    pub fn GetOwnPropertyNames(out: &mut Local, object: Local) -> bool;

    #[link_name = "NeonSys_Object_GetIsolate"]
    pub fn GetIsolate(obj: Local) -> *mut Isolate;

    #[link_name = "NeonSys_Object_Get_Index"]
    pub fn Get_Index(out: &mut Local, object: Local, index: u32) -> bool;

    #[link_name = "NeonSys_Object_Set_Index"]
    pub fn Set_Index(out: &mut bool, object: Local, index: u32, val: Local) -> bool;

    #[link_name = "NeonSys_Object_Get_String"]
    pub fn Get_String(out: &mut Local, object: Local, key: *const u8, len: i32) -> bool;

    #[link_name = "NeonSys_Object_Set_String"]
    pub fn Set_String(out: &mut bool, object: Local, key: *const u8, len: i32, val: Local) -> bool;

    #[link_name = "NeonSys_Object_Get"]
    pub fn Get(out: &mut Local, object: Local, key: Local) -> bool;

    #[link_name = "NeonSys_Object_Set"]
    pub fn Set(out: &mut bool, object: Local, key: Local, val: Local) -> bool;

}
