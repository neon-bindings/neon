//! Facilities for working with `v8::Object`s.

use raw::{Isolate, Local};

/// Mutates the `out` argument provided to refer to a newly created `v8::Object`.
pub use neon_sys::Neon_Object_New as new;

/// Mutates the `out` argument provided to refer to a newly created `v8::Array` containing the
/// names of the `v8::Object`'s own property names. Returns `false` if the result is empty.
pub use neon_sys::Neon_Object_GetOwnPropertyNames as get_own_property_names;

/// Gets the `v8::Isolate` of a `v8::Object`.
pub use neon_sys::Neon_Object_GetIsolate as get_isolate;

/// Mutates the `out` argument provided to refer to the `v8::Local` value at the `index`
/// provided of the `v8::Object`. Returns `false` if the result couldn't be retrieved.
pub use neon_sys::Neon_Object_Get_Index as get_index;

/// Sets the key value of a `v8::Object` at the `index` provided. Also mutates the `out`
/// argument provided to refer to a `v8::Local` boolean value, `true` if the set was
/// successful.
pub use neon_sys::Neon_Object_Set_Index as set_index;

/// Mutates the `out` argument provided to refer to the `v8::Local` value of the `v8::String`'s
/// underlying content.  Returns `false` if the value couldn't be retrieved.
pub use neon_sys::Neon_Object_Get_String as get_string;

/// Sets the underlying content of a `v8::String` object. Also mutates the `out` argument
/// provided to refer to a `v8::Local` boolean value, `true` if the set was successful.
pub unsafe extern "C" fn set_string(
    _: Isolate,
    out: &mut bool,
    object: Local,
    key: *const u8,
    len: i32,
    val: Local,
) -> bool {
    neon_sys::Neon_Object_Set_String(out, object, key, len, val)
}

/// Mutates the `out` argument provided to refer to the `v8::Local` value at the `key`
/// provided. Returns `false` if the result couldn't be retrieved.
pub use neon_sys::Neon_Object_Get as get;

/// Sets the key value of a `v8::Object` at the `key` provided. Also mutates the `out` argument
/// provided to refer to a `v8::Local` boolean value, `true` if the set was successful.
pub use neon_sys::Neon_Object_Set as set;
