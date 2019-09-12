//! Helper functions for converting `v8::Local` values.

use raw::Local;

// FIXME(napi): #[link_name = "Neon_Convert_ToObject"]
pub extern "C" fn to_object(out: &mut Local, value: &Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Convert_ToString"]
pub extern "C" fn to_string(out: &mut Local, value: Local) -> bool { unimplemented!() }
