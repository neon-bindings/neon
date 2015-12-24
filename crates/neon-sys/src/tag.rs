use raw::Local;

// analog C enum `tag_t` defined in neon.h
#[repr(C)]
#[derive(PartialEq, Eq)]
pub enum Tag {
    Null,
    Undefined,
    Boolean,
    Integer,
    Number,
    String,
    Object,
    Array,
    Function,
    Other
}

extern "system" {

    #[link_name = "NeonSys_Tag_Of"]
    pub fn Of(val: Local) -> Tag;

    #[link_name = "NeonSys_Tag_IsUndefined"]
    pub fn IsUndefined(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsNull"]
    pub fn IsNull(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsInteger"]
    pub fn IsInteger(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsNumber"]
    pub fn IsNumber(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsBoolean"]
    pub fn IsBoolean(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsString"]
    pub fn IsString(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsObject"]
    pub fn IsObject(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsArray"]
    pub fn IsArray(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsFunction"]
    pub fn IsFunction(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsTypeError"]
    pub fn IsTypeError(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsBuffer"]
    pub fn IsBuffer(obj: Local) -> bool;

}
