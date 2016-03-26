use raw::{Local, Isolate};

extern "system" {

    #[link_name = "NeonSys_Primitive_Undefined"]
    pub fn undefined(out: &mut Local);

    #[link_name = "NeonSys_Primitive_Null"]
    pub fn null(out: &mut Local);

    #[link_name = "NeonSys_Primitive_Boolean"]
    pub fn boolean(out: &mut Local, b: bool);

    #[link_name = "NeonSys_Primitive_BooleanValue"]
    pub fn boolean_value(p: Local) -> bool;

    #[link_name = "NeonSys_Primitive_Integer"]
    pub fn integer(out: &mut Local, isolate: *mut Isolate, x: i32);

    #[link_name = "NeonSys_Primitive_IsUint32"]
    pub fn is_u32(p: Local) -> bool;

    #[link_name = "NeonSys_Primitive_IsInt32"]
    pub fn is_i32(p: Local) -> bool;

    #[link_name = "NeonSys_Primitive_IntegerValue"]
    pub fn integer_value(p: Local) -> i64;

    #[link_name = "NeonSys_Primitive_Number"]
    pub fn number(out: &mut Local, isolate: *mut Isolate, v: f64);

    #[link_name = "NeonSys_Primitive_NumberValue"]
    pub fn number_value(p: Local) -> f64;
}
