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

extern "C" {

    #[link_name = "NeonSys_Tag_Of"]
    pub fn of(val: Local) -> Tag;

    #[link_name = "NeonSys_Tag_IsUndefined"]
    pub fn is_undefined(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsNull"]
    pub fn is_null(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsInteger"]
    pub fn is_integer(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsNumber"]
    pub fn is_number(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsBoolean"]
    pub fn is_boolean(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsString"]
    pub fn is_string(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsObject"]
    pub fn is_object(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsArray"]
    pub fn is_array(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsFunction"]
    pub fn is_function(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsError"]
    pub fn is_error(val: Local) -> bool;

    #[link_name = "NeonSys_Tag_IsBuffer"]
    pub fn is_buffer(obj: Local) -> bool;

}
