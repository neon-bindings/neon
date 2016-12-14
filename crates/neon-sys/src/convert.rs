//! Helper functions for converting `Local` values.

use raw::Local;

extern "system" {

    /// Casts the value provided to a `Object` and mutates the `out` argument provided to refer to
    /// `Local` handle of the converted value. Returns `false` if the conversion didn't succeed.
    #[link_name = "NeonSys_Convert_ToObject"]
    pub fn to_object(out: &mut Local, value: &Local) -> bool;

    /// Casts the value provided to a `String` and mutates the `out` argument provided to refer to
    /// `Local` handle of the converted value. Returns `false` if the conversion didn't succeed.
    #[link_name = "NeonSys_Convert_ToString"]
    pub fn to_string(out: &mut Local, value: Local) -> bool;

}
