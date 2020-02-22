//! Facilities for working with `v8::Function`s.

/// Mutates the `out` argument provided to refer to a newly created `v8::Function`. Returns
/// `false` if the value couldn't be created.
pub use neon_sys::Neon_Fun_New as new;

/// Mutates the `out` argument provided to refer to a newly created `v8::FunctionTemplate`.
/// Returns `false` if the value couldn't be created.
pub use neon_sys::Neon_Fun_Template_New as new_template;

/// Gets the reference to the `v8::Local<v8::External>` handle provided.
pub use neon_sys::Neon_Fun_GetDynamicCallback as get_dynamic_callback;

/// Calls the function provided (`fun`) and mutates the `out` argument provided to refer to the
/// result of the function call. Returns `false` if the result of the call was empty.
pub use neon_sys::Neon_Fun_Call as call;

/// Makes a constructor call to with the function provided (`fun`) and mutates the `out`
/// argument provided to refer to the result of the constructor call. Returns `false` if the
/// result of the call was empty.
pub use neon_sys::Neon_Fun_Construct as construct;
