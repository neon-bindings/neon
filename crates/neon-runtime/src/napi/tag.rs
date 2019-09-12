//! Facilities for identifying the type of a `v8::Local` handle.

use raw::Local;

// FIXME(napi): #[link_name = "Neon_Tag_IsUndefined"]
pub extern "C" fn is_undefined(val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Tag_IsNull"]
pub extern "C" fn is_null(val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Tag_IsNumber"]
pub extern "C" fn is_number(val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Tag_IsBoolean"]
pub extern "C" fn is_boolean(val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Tag_IsString"]
pub extern "C" fn is_string(val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Tag_IsObject"]
pub extern "C" fn is_object(val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Tag_IsArray"]
pub extern "C" fn is_array(val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Tag_IsFunction"]
pub extern "C" fn is_function(val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Tag_IsError"]
pub extern "C" fn is_error(val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Tag_IsBuffer"]
pub extern "C" fn is_buffer(obj: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Tag_IsArrayBuffer"]
pub extern "C" fn is_arraybuffer(obj: Local) -> bool { unimplemented!() }
