use raw::{Local, Isolate};

extern "system" {

    #[link_name = "NeonSys_Primitive_Undefined"]
    pub fn Undefined(out: &mut Local);

    #[link_name = "NeonSys_Primitive_Null"]
    pub fn Null(out: &mut Local);

    #[link_name = "NeonSys_Primitive_Boolean"]
    pub fn Boolean(out: &mut Local, b: bool);

    #[link_name = "NeonSys_Primitive_Integer"]
    pub fn Integer(out: &mut Local, isolate: *mut Isolate, x: i32);

    #[link_name = "NeonSys_Primitive_Number"]
    pub fn Number(out: &mut Local, isolate: *mut Isolate, v: f64);

}
