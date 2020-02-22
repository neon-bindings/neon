pub use neon_sys::Neon_Class_GetClassMap as get_class_map;

pub use neon_sys::Neon_Class_SetClassMap as set_class_map;

pub use neon_sys::Neon_Class_CreateBase as create_base;

pub use neon_sys::Neon_Class_GetName as get_name;

pub use neon_sys::Neon_Class_SetName as set_name;

pub use neon_sys::Neon_Class_ThrowCallError as throw_call_error;

pub use neon_sys::Neon_Class_ThrowThisError as throw_this_error;

pub use neon_sys::Neon_Class_AddMethod as add_method;

pub use neon_sys::Neon_Class_MetadataToConstructor as metadata_to_constructor;

// FIXME: get rid of all the "kernel" nomenclature

pub use neon_sys::Neon_Class_GetAllocateKernel as get_allocate_kernel;

pub use neon_sys::Neon_Class_GetConstructKernel as get_construct_kernel;

pub use neon_sys::Neon_Class_GetCallKernel as get_call_kernel;

pub use neon_sys::Neon_Class_Constructor as constructor;

pub use neon_sys::Neon_Class_HasInstance as has_instance;

pub use neon_sys::Neon_Class_GetInstanceInternals as get_instance_internals;
