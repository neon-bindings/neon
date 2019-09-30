//! Facilities for identifying the type of a `v8::Local` handle.

/// Indicates if the value type is `Undefined`.
pub use neon_sys::Neon_Tag_IsUndefined as is_undefined;

/// Indicates if the value type is `Null`.
pub use neon_sys::Neon_Tag_IsNull as is_null;

/// Indicates if the value type is `Number`.
pub use neon_sys::Neon_Tag_IsNumber as is_number;

/// Indicates if the value type is `Boolean`.
pub use neon_sys::Neon_Tag_IsBoolean as is_boolean;

/// Indicates if the value type is `String`.
pub use neon_sys::Neon_Tag_IsString as is_string;

/// Indicates if the value type is `Object`.
pub use neon_sys::Neon_Tag_IsObject as is_object;

/// Indicates if the value type is `Array`.
pub use neon_sys::Neon_Tag_IsArray as is_array;

/// Indicates if the value type is `Function`.
pub use neon_sys::Neon_Tag_IsFunction as is_function;

/// Indicates if the value type is `Error`.
pub use neon_sys::Neon_Tag_IsError as is_error;

/// Indicates if the value type is `Buffer`.
pub use neon_sys::Neon_Tag_IsBuffer as is_buffer;

/// Indicates if the value type is `ArrayBuffer`.
pub use neon_sys::Neon_Tag_IsArrayBuffer as is_arraybuffer;
