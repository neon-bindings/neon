//! Facilities for working with `v8::FunctionCallbackInfo` and getting the current `v8::Isolate`.

pub use neon_sys::CCallback;

/// Sets the return value of the function call.
pub use neon_sys::Neon_Call_SetReturn as set_return;

/// Gets the isolate of the function call.
pub use neon_sys::Neon_Call_GetIsolate as get_isolate;

/// Gets the current `v8::Isolate`.
pub use neon_sys::Neon_Call_CurrentIsolate as current_isolate;

/// Indicates if the function call was invoked as a constructor.
pub use neon_sys::Neon_Call_IsConstruct as is_construct;

/// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the object
/// the function is bound to.
pub use neon_sys::Neon_Call_This as this;

/// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the
/// `v8::FunctionCallbackInfo` `Data`.
pub use neon_sys::Neon_Call_Data as data;

/// Gets the number of arguments passed to the function.
pub use neon_sys::Neon_Call_Length as len;

/// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the `i`th
/// argument passed to the function.
pub use neon_sys::Neon_Call_Get as get;
