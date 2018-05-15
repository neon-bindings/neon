use std::os::raw::c_void;
use call::CCallback;
use raw::{Isolate, Local};

extern "C" {

    #[link_name = "Neon_Class_GetClassMap"]
    pub fn get_class_map(isolate: *mut Isolate) -> *mut c_void;

    #[link_name = "Neon_Class_SetClassMap"]
    pub fn set_class_map(isolate: *mut Isolate, map: *mut c_void, free_map: *mut c_void);

    #[link_name = "Neon_Class_CreateBase"]
    pub fn create_base(isolate: *mut Isolate,
                       allocate: CCallback,
                       construct: CCallback,
                       call: CCallback,
                       drop: extern "C" fn(*mut c_void)) -> *mut c_void;

    #[link_name = "Neon_Class_SetName"]
    pub fn set_name(isolate: *mut Isolate, metadata: *mut c_void, name: *const u8, byte_length: u32) -> bool;

    #[link_name = "Neon_Class_ThrowCallError"]
    pub fn throw_call_error(isolate: *mut Isolate, metadata: *mut c_void);

    #[link_name = "Neon_Class_ThrowThisError"]
    pub fn throw_this_error(isolate: *mut Isolate, metadata: *mut c_void);

    #[link_name = "Neon_Class_AddMethod"]
    pub fn add_method(isolate: *mut Isolate, metadata: *mut c_void, name: *const u8, byte_length: u32, method: Local) -> bool;

    #[link_name = "Neon_Class_MetadataToClass"]
    pub fn metadata_to_class(out: &mut Local, isolate: *mut Isolate, metadata: *mut c_void);

    // FIXME: get rid of all the "kernel" nomenclature

    #[link_name = "Neon_Class_GetAllocateKernel"]
    pub fn get_allocate_kernel(obj: Local) -> *mut c_void;

    #[link_name = "Neon_Class_GetConstructKernel"]
    pub fn get_construct_kernel(obj: Local) -> *mut c_void;

    #[link_name = "Neon_Class_GetCallKernel"]
    pub fn get_call_kernel(obj: Local) -> *mut c_void;

    #[link_name = "Neon_Class_Constructor"]
    pub fn constructor(out: &mut Local, ft: Local) -> bool;

    #[link_name = "Neon_Class_Check"]
    pub fn check(c: Local, v: Local) -> bool;

    #[link_name = "Neon_Class_HasInstance"]
    pub fn has_instance(metadata: *mut c_void, v: Local) -> bool;

    #[link_name = "Neon_Class_GetInstanceInternals"]
    pub fn get_instance_internals(obj: Local) -> *mut c_void;

}
