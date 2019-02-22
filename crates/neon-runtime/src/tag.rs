//! Facilities for identifying the type of a `v8::Local` handle.

use raw::Persistent;

extern "C" {

    /// Indicates if the value type is `Undefined`.
    #[link_name = "Neon_Tag_IsUndefined"]
    pub fn is_undefined(val: &Persistent) -> bool;

    /// Indicates if the value type is `Null`.
    #[link_name = "Neon_Tag_IsNull"]
    pub fn is_null(val: &Persistent) -> bool;

    /// Indicates if the value type is `Number`.
    #[link_name = "Neon_Tag_IsNumber"]
    pub fn is_number(val: &Persistent) -> bool;

    /// Indicates if the value type is `Boolean`.
    #[link_name = "Neon_Tag_IsBoolean"]
    pub fn is_boolean(val: &Persistent) -> bool;

    /// Indicates if the value type is `String`.
    #[link_name = "Neon_Tag_IsString"]
    pub fn is_string(val: &Persistent) -> bool;

    /// Indicates if the value type is `Object`.
    #[link_name = "Neon_Tag_IsObject"]
    pub fn is_object(val: &Persistent) -> bool;

    /// Indicates if the value type is `Array`.
    #[link_name = "Neon_Tag_IsArray"]
    pub fn is_array(val: &Persistent) -> bool;

    /// Indicates if the value type is `Function`.
    #[link_name = "Neon_Tag_IsFunction"]
    pub fn is_function(val: &Persistent) -> bool;

    /// Indicates if the value type is `Error`.
    #[link_name = "Neon_Tag_IsError"]
    pub fn is_error(val: &Persistent) -> bool;

    /// Indicates if the value type is `Buffer`.
    #[link_name = "Neon_Tag_IsBuffer"]
    pub fn is_buffer(val: &Persistent) -> bool;

    /// Indicates if the value type is `ArrayBuffer`.
    #[link_name = "Neon_Tag_IsArrayBuffer"]
    pub fn is_arraybuffer(val: &Persistent) -> bool;

}
