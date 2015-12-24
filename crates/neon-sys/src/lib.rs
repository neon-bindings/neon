pub mod raw;
pub mod buf;

use std::os::raw::c_void;
use raw::{FunctionCallbackInfo, EscapableHandleScope, Isolate, Local};
use buf::Buf;

// analog C enum `tag_t` defined in neon.h
#[repr(C)]
#[derive(PartialEq, Eq)]
pub enum Tag {
    Null,
    Undefined,
    Boolean,
    Integer,
    Number,
    String,
    Object,
    Array,
    Function,
    Other
}

extern "system" {
    // Function call activations

    pub fn NeonSys_Call_SetReturn(info: &FunctionCallbackInfo, value: Local);
    pub fn NeonSys_Call_GetIsolate(info: &FunctionCallbackInfo) -> &Isolate;
    pub fn NeonSys_Call_IsConstruct(info: &FunctionCallbackInfo) -> bool;
    pub fn NeonSys_Call_This(info: &FunctionCallbackInfo, out: &mut Local);
    pub fn NeonSys_Call_Callee(info: &FunctionCallbackInfo, out: &mut Local);
    pub fn NeonSys_Call_Data(info: &FunctionCallbackInfo, out: &mut Local);
    pub fn NeonSys_Call_Length(info: &FunctionCallbackInfo) -> i32;
    pub fn NeonSys_Call_Get(info: &FunctionCallbackInfo, i: i32, out: &mut Local);


    // Scopes

    pub fn NeonSys_Escape(out: &mut Local, scope: *mut EscapableHandleScope, value: Local);
    pub fn NeonSys_Chained(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void), parent_scope: *mut c_void);
    pub fn NeonSys_Nested(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void), realm: *mut c_void);


    // Allocators

    pub fn NeonSys_NewObject(out: &mut Local);
    pub fn NeonSys_NewUndefined(out: &mut Local);
    pub fn NeonSys_NewNull(out: &mut Local);
    pub fn NeonSys_NewBoolean(out: &mut Local, b: bool);
    pub fn NeonSys_NewInteger(out: &mut Local, isolate: *mut Isolate, x: i32);
    pub fn NeonSys_NewString(out: &mut Local, isolate: *mut Isolate, data: *const u8, len: i32) -> bool;
    pub fn NeonSys_NewNumber(out: &mut Local, isolate: *mut Isolate, v: f64);
    pub fn NeonSys_NewArray(out: &mut Local, isolate: *mut Isolate, length: u32);
    pub fn NeonSys_NewBuffer(out: &mut Local, size: u32) -> bool;
    pub fn NeonSys_NewFunction(out: &mut Local, isolate: *mut c_void, callback: *mut c_void, kernel: *mut c_void) -> bool;
    pub fn NeonSys_NewTypeError(out: &mut Local, msg: *const u8) -> bool;


    // Handles

    pub fn NeonSys_SameHandle(h1: Local, h2: Local) -> bool;


    // Objects

    pub fn NeonSys_Object_GetOwnPropertyNames(out: &mut Local, object: Local) -> bool;
    pub fn NeonSys_Object_GetIsolate(obj: Local) -> *mut Isolate;
    pub fn NeonSys_Object_Get_Index(out: &mut Local, object: Local, index: u32) -> bool;
    pub fn NeonSys_Object_Set_Index(out: &mut bool, object: Local, index: u32, val: Local) -> bool;
    pub fn NeonSys_Object_Get_String(out: &mut Local, object: Local, key: *const u8, len: i32) -> bool;
    pub fn NeonSys_Object_Set_String(out: &mut bool, object: Local, key: *const u8, len: i32, val: Local) -> bool;
    pub fn NeonSys_Object_Get(out: &mut Local, object: Local, key: Local) -> bool;
    pub fn NeonSys_Object_Set(out: &mut bool, object: Local, key: Local, val: Local) -> bool;


    // Strings

    pub fn NeonSys_String_Utf8Length(str: Local) -> isize;
    pub fn NeonSys_String_Data(out: *mut u8, len: isize, str: Local) -> isize;


    // Arrays

    pub fn NeonSys_Array_Length(array: Local) -> u32;


    // Conversions

    pub fn NeonSys_Value_ToObject(out: &mut Local, value: &Local) -> bool;
    pub fn NeonSys_Value_ToString(out: &mut Local, value: Local) -> bool;


    // Typed arrays

    pub fn NeonSys_Buffer_Data<'a, 'b>(out: &'a mut Buf<'b>, obj: Local);


    // Internal callbacks

    pub fn NeonSys_ExecFunctionBody(closure: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void), info: &FunctionCallbackInfo, scope: *mut c_void);
    pub fn NeonSys_ExecModuleBody(kernel: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void), exports: Local, scope: *mut c_void);
    pub fn NeonSys_FunctionKernel(obj: Local) -> *mut c_void;


    // Value tags

    pub fn NeonSys_TagOf(val: Local) -> Tag;
    pub fn NeonSys_IsUndefined(val: Local) -> bool;
    pub fn NeonSys_IsNull(val: Local) -> bool;
    pub fn NeonSys_IsInteger(val: Local) -> bool;
    pub fn NeonSys_IsNumber(val: Local) -> bool;
    pub fn NeonSys_IsBoolean(val: Local) -> bool;
    pub fn NeonSys_IsString(val: Local) -> bool;
    pub fn NeonSys_IsObject(val: Local) -> bool;
    pub fn NeonSys_IsArray(val: Local) -> bool;
    pub fn NeonSys_IsFunction(val: Local) -> bool;
    pub fn NeonSys_IsTypeError(val: Local) -> bool;
    pub fn NeonSys_IsBuffer(obj: Local) -> bool;


    // Errors

    pub fn NeonSys_ThrowAny(val: Local);
    pub fn NeonSys_ThrowTypeError(msg: *const u8);
}
