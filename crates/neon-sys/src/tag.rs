//! Facilities for identifying the type of a `v8::Local` handle.

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

    /// Returns the `Tag` of the value provided.
    #[link_name = "NeonSys_Tag_Of"]
    pub fn of(val: Local) -> Tag;

    /// Indicates if the value type is `Undefined`.
    #[link_name = "NeonSys_Tag_IsUndefined"]
    pub fn is_undefined(val: Local) -> bool;

    /// Indicates if the value type is `Null`.
    #[link_name = "NeonSys_Tag_IsNull"]
    pub fn is_null(val: Local) -> bool;

    /// Indicates if the value type is `Integer`.
    #[link_name = "NeonSys_Tag_IsInteger"]
    pub fn is_integer(val: Local) -> bool;

    /// Indicates if the value type is `Number`.
    #[link_name = "NeonSys_Tag_IsNumber"]
    pub fn is_number(val: Local) -> bool;

    /// Indicates if the value type is `Boolean`.
    #[link_name = "NeonSys_Tag_IsBoolean"]
    pub fn is_boolean(val: Local) -> bool;

    /// Indicates if the value type is `String`.
    #[link_name = "NeonSys_Tag_IsString"]
    pub fn is_string(val: Local) -> bool;

    /// Indicates if the value type is `Object`.
    #[link_name = "NeonSys_Tag_IsObject"]
    pub fn is_object(val: Local) -> bool;

    /// Indicates if the value type is `Array`.
    #[link_name = "NeonSys_Tag_IsArray"]
    pub fn is_array(val: Local) -> bool;

    /// Indicates if the value type is `Function`.
    #[link_name = "NeonSys_Tag_IsFunction"]
    pub fn is_function(val: Local) -> bool;

    /// Indicates if the value type is `Error`.
    #[link_name = "NeonSys_Tag_IsError"]
    pub fn is_error(val: Local) -> bool;

    /// Indicates if the value type is `Buffer`.
    #[link_name = "NeonSys_Tag_IsBuffer"]
    pub fn is_buffer(obj: Local) -> bool;

}
