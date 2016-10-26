//! Helper functions for converting `v8::Local` values.

use raw::Local;

extern "C" {

    /// Casts the value provided to a `v8::Object` and mutates the `out` argument provided to refer
    /// to `v8::Local` handle of the converted value. Returns `false` if the conversion didn't
    /// succeed.
    #[link_name = "NeonSys_Convert_ToObject"]
    pub fn to_object(out: &mut Local, value: &Local) -> bool;

    /// Casts the value provided to a `v8::String` and mutates the `out` argument provided to refer
    /// to `v8::Local` handle of the converted value. Returns `false` if the conversion didn't
    /// succeed.
    #[link_name = "NeonSys_Convert_ToString"]
    pub fn to_string(out: &mut Local, value: Local) -> bool;

}
