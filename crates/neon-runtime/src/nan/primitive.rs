//! Facilities for working with primitive values.

/// Mutates the `out` argument provided to refer to the `v8::Undefined` object.
pub use neon_sys::Neon_Primitive_Undefined as undefined;

/// Mutates the `out` argument provided to refer to the `v8::Null` object.
pub use neon_sys::Neon_Primitive_Null as null;

/// Mutates the `out` argument provided to refer to the `v8::Boolean` object.
pub use neon_sys::Neon_Primitive_Boolean as boolean;

/// Gets the underlying value of a `v8::Boolean` object.
pub use neon_sys::Neon_Primitive_BooleanValue as boolean_value;

// DEPRECATE(0.2)
/// Mutates the `out` argument provided to refer to a newly created `v8::Integer` object.
pub use neon_sys::Neon_Primitive_Integer as integer;

/// Indicates if the value is a 32-bit unsigned integer.
pub use neon_sys::Neon_Primitive_IsUint32 as is_u32;

/// Indicates if the value is a 32-bit signed integer.
pub use neon_sys::Neon_Primitive_IsInt32 as is_i32;

// DEPRECATE(0.2)
/// Gets the underlying value of a `v8::Integer` object.
pub use neon_sys::Neon_Primitive_IntegerValue as integer_value;

/// Mutates the `out` argument provided to refer to a newly created `v8::Number` object.
pub use neon_sys::Neon_Primitive_Number as number;

/// Gets the underlying value of a `v8::Number` object.
pub use neon_sys::Neon_Primitive_NumberValue as number_value;

pub use neon_sys::Neon_Primitive_BigInt as bigint;

pub use neon_sys::Neon_Primitive_BigIntValue as bigint_value;
