use std::os::raw::c_void;
use call::CCallback;
use raw::{Isolate, Local};

// FIXME(napi): #[link_name = "Neon_Class_GetClassMap"]
pub extern "C" fn get_class_map(isolate: *mut Isolate) -> *mut c_void { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_SetClassMap"]
pub extern "C" fn set_class_map(isolate: *mut Isolate, map: *mut c_void, free_map: *mut c_void) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_CreateBase"]
pub extern "C" fn create_base(isolate: *mut Isolate,
                              allocate: CCallback,
                              construct: CCallback,
                              call: CCallback,
                              drop: extern "C" fn(*mut c_void)) -> *mut c_void { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_GetName"]
pub extern "C" fn get_name<'a>(base_out: &'a mut *mut u8, isolate: *mut Isolate, metadata: *const c_void) -> usize { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_SetName"]
pub extern "C" fn set_name(isolate: *mut Isolate, metadata: *mut c_void, name: *const u8, byte_length: u32) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_ThrowCallError"]
pub extern "C" fn throw_call_error(isolate: *mut Isolate, metadata: *mut c_void) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_ThrowThisError"]
pub extern "C" fn throw_this_error(isolate: *mut Isolate, metadata: *mut c_void) { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_AddMethod"]
pub extern "C" fn add_method(isolate: *mut Isolate, metadata: *mut c_void, name: *const u8, byte_length: u32, method: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_MetadataToConstructor"]
pub extern "C" fn metadata_to_constructor(out: &mut Local, isolate: *mut Isolate, metadata: *mut c_void) -> bool { unimplemented!() }

// FIXME: get rid of all the "kernel" nomenclature

// FIXME(napi): #[link_name = "Neon_Class_GetAllocateKernel"]
pub extern "C" fn get_allocate_kernel(obj: Local) -> *mut c_void { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_GetConstructKernel"]
pub extern "C" fn get_construct_kernel(obj: Local) -> *mut c_void { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_GetCallKernel"]
pub extern "C" fn get_call_kernel(obj: Local) -> *mut c_void { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_Constructor"]
pub extern "C" fn constructor(out: &mut Local, ft: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_HasInstance"]
pub extern "C" fn has_instance(metadata: *mut c_void, v: Local) -> bool { unimplemented!() }

// FIXME(napi): #[link_name = "Neon_Class_GetInstanceInternals"]
pub extern "C" fn get_instance_internals(obj: Local) -> *mut c_void { unimplemented!() }
