//! Facilities for working with `v8::Object`s.

use raw::{Isolate, Local};

// FIXME(napi): #[link_name = "Neon_Object_New"]
pub extern "C" fn new(out: &mut Local) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Object_GetOwnPropertyNames"]
pub extern "C" fn get_own_property_names(out: &mut Local, object: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Object_GetIsolate"]
pub extern "C" fn get_isolate(obj: Local) -> *mut Isolate { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Object_Get_Index"]
pub extern "C" fn get_index(out: &mut Local, object: Local, index: u32) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Object_Set_Index"]
pub extern "C" fn set_index(out: &mut bool, object: Local, index: u32, val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Object_Get_String"]
pub extern "C" fn get_string(out: &mut Local, object: Local, key: *const u8, len: i32) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Object_Set_String"]
pub extern "C" fn set_string(out: &mut bool, object: Local, key: *const u8, len: i32, val: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Object_Get"]
pub extern "C" fn get(out: &mut Local, object: Local, key: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Object_Set"]
pub extern "C" fn set(out: &mut bool, object: Local, key: Local, val: Local) -> bool { unimplemented!() }
