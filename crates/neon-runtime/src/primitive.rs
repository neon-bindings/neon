//! Facilities for working with primitive values.

use raw::{Local, Isolate, Persistent};

extern "C" {

    /// Mutates the `out` argument provided to refer to the `v8::Undefined` object.
    #[link_name = "Neon_Primitive_Undefined"]
    pub fn undefined(out: &mut Local);

    /// Initializes the `out` argument with a newly created `v8::Undefined` object.
    #[link_name = "Neon_Primitive_InitUndefined"]
    pub fn init_undefined(out: &Persistent, isolate: *mut Isolate);

    /// Mutates the `out` argument provided to refer to the `v8::Null` object.
    #[link_name = "Neon_Primitive_InitNull"]
    pub fn init_null(out: &Persistent, isolate: *mut Isolate);

    /// Initializes the `out` argument with a newly created `v8::Boolean` object.
    #[link_name = "Neon_Primitive_InitBoolean"]
    pub fn init_boolean(out: &Persistent, isolate: *mut Isolate, b: bool);

    /// Gets the underlying value of a `v8::Boolean` object.
    #[link_name = "Neon_Primitive_BooleanValue"]
    pub fn boolean_value(p: &Persistent) -> bool;

    // DEPRECATE(0.2)
    /// Mutates the `out` argument provided to refer to a newly created `v8::Integer` object.
    #[link_name = "Neon_Primitive_Integer"]
    pub fn integer(out: &mut Local, isolate: *mut Isolate, x: i32);

    /// Indicates if the value is a 32-bit unsigned integer.
    #[link_name = "Neon_Primitive_IsUint32"]
    pub fn is_u32(p: Local) -> bool;

    /// Indicates if the value is a 32-bit signed integer.
    #[link_name = "Neon_Primitive_IsInt32"]
    pub fn is_i32(p: Local) -> bool;

    // DEPRECATE(0.2)
    /// Gets the underlying value of a `v8::Integer` object.
    #[link_name = "Neon_Primitive_IntegerValue"]
    pub fn integer_value(p: Local) -> i64;

    /// Initializes the `out` argument with a newly created `v8::Number` object.
    #[link_name = "Neon_Primitive_InitNumber"]
    pub fn init_number(out: &Persistent, isolate: *mut Isolate, v: f64);

    /// Gets the underlying value of a `v8::Number` object.
    #[link_name = "Neon_Primitive_NumberValue"]
    pub fn number_value(p: &Persistent) -> f64;
}
