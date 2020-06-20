use std::os::raw::c_void;
use call::CCallback;
use raw::{Env, Local};

pub unsafe extern "C" fn get_class_map(_isolate: Env) -> *mut c_void { unimplemented!() }

pub unsafe extern "C" fn set_class_map(_isolate: Env, _map: *mut c_void, _free_map: *mut c_void) { unimplemented!() }

pub unsafe extern "C" fn create_base(_isolate: Env,
                                     _allocate: CCallback,
                                     _construct: CCallback,
                                     _call: CCallback,
                                     _drop: extern "C" fn(*mut c_void)) -> *mut c_void { unimplemented!() }

pub unsafe extern "C" fn get_name<'a>(_base_out: &'a mut *mut u8, _isolate: Env, _metadata: *const c_void) -> usize { unimplemented!() }

pub unsafe extern "C" fn set_name(_isolate: Env, _metadata: *mut c_void, _name: *const u8, _byte_length: u32) -> bool { unimplemented!() }

pub unsafe extern "C" fn throw_call_error(_isolate: Env, _metadata: *mut c_void) { unimplemented!() }

pub unsafe extern "C" fn throw_this_error(_isolate: Env, _metadata: *mut c_void) { unimplemented!() }

pub unsafe extern "C" fn add_method(_isolate: Env, _metadata: *mut c_void, _name: *const u8, _byte_length: u32, _method: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn metadata_to_constructor(_out: &mut Local, _isolate: Env, _metadata: *mut c_void) -> bool { unimplemented!() }

// FIXME: get rid of all the "kernel" nomenclature

pub unsafe extern "C" fn get_allocate_kernel(_data: *mut c_void) -> *mut c_void { unimplemented!() }

pub unsafe extern "C" fn get_construct_kernel(_data: *mut c_void) -> *mut c_void { unimplemented!() }

pub unsafe extern "C" fn get_call_kernel(_data: *mut c_void) -> *mut c_void { unimplemented!() }

pub unsafe extern "C" fn constructor(_out: &mut Local, _ft: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn has_instance(_metadata: *mut c_void, _v: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn get_instance_internals(_obj: Local) -> *mut c_void { unimplemented!() }
