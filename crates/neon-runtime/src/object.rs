//! Facilities for working with `v8::Object`s.

use raw::{Isolate, Local};

extern "C" {

    /// Mutates the `out` argument provided to refer to a newly created `v8::Object`.
    #[link_name = "NeonSys_Object_New"]
    pub fn new(out: &mut Local);

    /// Mutates the `out` argument provided to refer to a newly created `v8::Array` containing the
    /// names of the `v8::Object`'s own property names. Returns `false` if the result is empty.
    #[link_name = "NeonSys_Object_GetOwnPropertyNames"]
    pub fn get_own_property_names(out: &mut Local, object: Local) -> bool;

    /// Gets the `v8::Isolate` of a `v8::Object`.
    #[link_name = "NeonSys_Object_GetIsolate"]
    pub fn get_isolate(obj: Local) -> *mut Isolate;

    /// Mutates the `out` argument provided to refer to the `v8::Local` value at the `index`
    /// provided of the `v8::Object`. Returns `false` if the result couldn't be retrieved.
    #[link_name = "NeonSys_Object_Get_Index"]
    pub fn get_index(out: &mut Local, object: Local, index: u32) -> bool;

    /// Sets the key value of a `v8::Object` at the `index` provided. Also mutates the `out`
    /// argument provided to refer to a `v8::Local` boolean value, `true` if the set was
    /// successful.
    #[link_name = "NeonSys_Object_Set_Index"]
    pub fn set_index(out: &mut bool, object: Local, index: u32, val: Local) -> bool;

    /// Mutates the `out` argument provided to refer to the `v8::Local` value of the `v8::String`'s
    /// underlying content.  Returns `false` if the value couldn't be retrieved.
    #[link_name = "NeonSys_Object_Get_String"]
    pub fn get_string(out: &mut Local, object: Local, key: *const u8, len: i32) -> bool;

    /// Sets the underlying content of a `v8::String` object. Also mutates the `out` argument
    /// provided to refer to a `v8::Local` boolean value, `true` if the set was successful.
    #[link_name = "NeonSys_Object_Set_String"]
    pub fn set_string(out: &mut bool, object: Local, key: *const u8, len: i32, val: Local) -> bool;

    /// Mutates the `out` argument provided to refer to the `v8::Local` value at the `key`
    /// provided. Returns `false` if the result couldn't be retrieved.
    #[link_name = "NeonSys_Object_Get"]
    pub fn get(out: &mut Local, object: Local, key: Local) -> bool;

    /// Sets the key value of a `v8::Object` at the `key` provided. Also mutates the `out` argument
    /// provided to refer to a `v8::Local` boolean value, `true` if the set was successful.
    #[link_name = "NeonSys_Object_Set"]
    pub fn set(out: &mut bool, object: Local, key: Local, val: Local) -> bool;

}
