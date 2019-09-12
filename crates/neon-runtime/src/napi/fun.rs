//! Facilities for working with `v8::Function`s.

use std::os::raw::c_void;
use call::CCallback;
use raw::Local;

// FIXME(napi): #[link_name = "Neon_Fun_New"]
pub extern "C" fn new(out: &mut Local, isolate: *mut c_void, callback: CCallback) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Fun_Template_New"]
pub extern "C" fn new_template(out: &mut Local, isolate: *mut c_void, callback: CCallback) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Fun_GetDynamicCallback"]
pub extern "C" fn get_dynamic_callback(obj: Local) -> *mut c_void { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Fun_Call"]
pub extern "C" fn call(out: &mut Local, isolate: *mut c_void, fun: Local, this: Local, argc: i32, argv: *mut c_void) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Fun_Construct"]
pub extern "C" fn construct(out: &mut Local, isolate: *mut c_void, fun: Local, argc: i32, argv: *mut c_void) -> bool { unimplemented!() }
