use std::os::raw::c_void;
use call::CCallback;
use raw::{Isolate, Local};

pub extern "C" fn get_class_map(isolate: *mut Isolate) -> *mut c_void { unimplemented!() }

pub extern "C" fn set_class_map(isolate: *mut Isolate, map: *mut c_void, free_map: *mut c_void) { unimplemented!() }

pub extern "C" fn create_base(isolate: *mut Isolate,
                              allocate: CCallback,
                              construct: CCallback,
                              call: CCallback,
                              drop: extern "C" fn(*mut c_void)) -> *mut c_void { unimplemented!() }

pub extern "C" fn get_name<'a>(base_out: &'a mut *mut u8, isolate: *mut Isolate, metadata: *const c_void) -> usize { unimplemented!() }

pub extern "C" fn set_name(isolate: *mut Isolate, metadata: *mut c_void, name: *const u8, byte_length: u32) -> bool { unimplemented!() }

pub extern "C" fn throw_call_error(isolate: *mut Isolate, metadata: *mut c_void) { unimplemented!() }

pub extern "C" fn throw_this_error(isolate: *mut Isolate, metadata: *mut c_void) { unimplemented!() }

pub extern "C" fn add_method(isolate: *mut Isolate, metadata: *mut c_void, name: *const u8, byte_length: u32, method: Local) -> bool { unimplemented!() }

pub extern "C" fn metadata_to_constructor(out: &mut Local, isolate: *mut Isolate, metadata: *mut c_void) -> bool { unimplemented!() }

// FIXME: get rid of all the "kernel" nomenclature

pub extern "C" fn get_allocate_kernel(obj: Local) -> *mut c_void { unimplemented!() }

pub extern "C" fn get_construct_kernel(obj: Local) -> *mut c_void { unimplemented!() }

pub extern "C" fn get_call_kernel(obj: Local) -> *mut c_void { unimplemented!() }

pub extern "C" fn constructor(out: &mut Local, ft: Local) -> bool { unimplemented!() }

pub extern "C" fn has_instance(metadata: *mut c_void, v: Local) -> bool { unimplemented!() }

pub extern "C" fn get_instance_internals(obj: Local) -> *mut c_void { unimplemented!() }
