use std::os::raw::c_void;
use raw::{Isolate, Local};

extern "system" {

    #[link_name = "NeonSys_Class_GetClassMap"]
    pub fn get_class_map(isolate: *mut Isolate) -> *mut c_void;

    #[link_name = "NeonSys_Class_SetClassMap"]
    pub fn set_class_map(isolate: *mut Isolate, map: *mut c_void, free_map: *mut c_void);

    #[link_name = "NeonSys_Class_CreateBase"]
    pub fn create_base(isolate: *mut Isolate,
                       allocate_callback: *mut c_void, allocate_kernel: *mut c_void,
                       construct_callback: *mut c_void, construct_kernel: *mut c_void,
                       call_callback: *mut c_void, call_kernel: *mut c_void,
                       drop: extern "C" fn(*mut c_void)) -> *mut c_void;

    #[link_name = "NeonSys_Class_SetName"]
    pub fn set_name(isolate: *mut Isolate, metadata: *mut c_void, name: *const u8, byte_length: u32) -> bool;

    #[link_name = "NeonSys_Class_ThrowCallError"]
    pub fn throw_call_error(isolate: *mut Isolate, metadata: *mut c_void);

    #[link_name = "NeonSys_Class_ThrowThisError"]
    pub fn throw_this_error(isolate: *mut Isolate, metadata: *mut c_void);

    #[link_name = "NeonSys_Class_AddMethod"]
    pub fn add_method(isolate: *mut Isolate, metadata: *mut c_void, name: *const u8, byte_length: u32, method: Local) -> bool;

    #[link_name = "NeonSys_Class_MetadataToClass"]
    pub fn metadata_to_class(out: &mut Local, isolate: *mut Isolate, metadata: *mut c_void);

    #[link_name = "NeonSys_Class_MetadataToInstance"]
    pub fn metadata_to_instance(out: &mut Local, isolate: *mut Isolate, metadata: *mut c_void, internals: *mut c_void);

    #[link_name = "NeonSys_Class_GetAllocateKernel"]
    pub fn get_allocate_kernel(obj: Local) -> *mut c_void;

    #[link_name = "NeonSys_Class_GetConstructKernel"]
    pub fn get_construct_kernel(obj: Local) -> *mut c_void;

    #[link_name = "NeonSys_Class_GetCallKernel"]
    pub fn get_call_kernel(obj: Local) -> *mut c_void;

    #[link_name = "NeonSys_Class_Constructor"]
    pub fn constructor(out: &mut Local, ft: Local) -> bool;

    #[link_name = "NeonSys_Class_Check"]
    pub fn check(c: Local, v: Local) -> bool;

    #[link_name = "NeonSys_Class_HasInstance"]
    pub fn has_instance(metadata: *mut c_void, v: Local) -> bool;

    #[link_name = "NeonSys_Class_GetInstanceInternals"]
    pub fn get_instance_internals(obj: Local) -> *mut c_void;

}
