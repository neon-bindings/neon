//! Facilities for working with primitive values.

use raw::{Isolate, Persistent};

extern "C" {

    /// Initializes the `out` argument with a newly created `v8::Undefined` object.
    #[link_name = "Neon_Primitive_Undefined"]
    pub fn undefined(out: &Persistent, isolate: *mut Isolate);

    /// Mutates the `out` argument provided to refer to the `v8::Null` object.
    #[link_name = "Neon_Primitive_Null"]
    pub fn null(out: &Persistent, isolate: *mut Isolate);

    /// Initializes the `out` argument with a newly created `v8::Boolean` object.
    #[link_name = "Neon_Primitive_Boolean"]
    pub fn boolean(out: &Persistent, isolate: *mut Isolate, b: bool);

    /// Gets the underlying value of a `v8::Boolean` object.
    #[link_name = "Neon_Primitive_BooleanValue"]
    pub fn boolean_value(p: &Persistent) -> bool;

    /// Initializes the `out` argument with a newly created `v8::Number` object.
    #[link_name = "Neon_Primitive_Number"]
    pub fn number(out: &Persistent, isolate: *mut Isolate, v: f64);

    /// Gets the underlying value of a `v8::Number` object.
    #[link_name = "Neon_Primitive_NumberValue"]
    pub fn number_value(p: &Persistent) -> f64;

}
