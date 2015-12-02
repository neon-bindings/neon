pub mod raw;
pub mod buf;

use std::os::raw::c_void;
use raw::{FunctionCallbackInfo, EscapableHandleScope, Isolate, Local};
use buf::Buf;

// analog C enum `tag_t` defined in nanny.h
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
    pub fn Nan_FunctionCallbackInfo_SetReturnValue(info: &FunctionCallbackInfo, value: Local);
    pub fn Nan_FunctionCallbackInfo_GetIsolate(info: &FunctionCallbackInfo) -> &Isolate;
    pub fn Nan_FunctionCallbackInfo_IsConstructCall(info: &FunctionCallbackInfo) -> bool;
    pub fn Nan_FunctionCallbackInfo_This(info: &FunctionCallbackInfo, out: &mut Local);
    pub fn Nan_FunctionCallbackInfo_Callee(info: &FunctionCallbackInfo, out: &mut Local);
    pub fn Nan_FunctionCallbackInfo_Data(info: &FunctionCallbackInfo, out: &mut Local);
    pub fn Nan_FunctionCallbackInfo_Length(info: &FunctionCallbackInfo) -> i32;
    pub fn Nan_FunctionCallbackInfo_Get(info: &FunctionCallbackInfo, i: i32, out: &mut Local);

    pub fn Nan_EscapableHandleScope_Escape(out: &mut Local, scope: *mut EscapableHandleScope, value: Local);

    pub fn Nan_NewObject(out: &mut Local);
    pub fn Nan_GetOwnPropertyNames(out: &mut Local, object: &Local) -> bool;
    pub fn Nan_Object_GetIsolate(obj: &Local) -> &Isolate;

    pub fn Nan_NewUndefined(out: &mut Local);
    pub fn Nan_NewNull(out: &mut Local);
    pub fn Nan_NewBoolean(out: &mut Local, b: bool);
    pub fn Nan_NewInteger(out: &mut Local, isolate: *mut Isolate, x: i32);
    pub fn Nan_NewString(out: &mut Local, isolate: *mut Isolate, data: *const u8, len: i32) -> bool;
    pub fn Nan_NewNumber(out: &mut Local, isolate: *mut Isolate, v: f64);
    pub fn Nan_NewArray(out: &mut Local, isolate: *mut Isolate, length: u32);
    pub fn Node_ArraySet(array: &mut Local, index: u32, value: Local) -> bool;
    pub fn Nan_Get_Index(out: &mut Local, object: &mut Local, index: u32) -> bool;
    pub fn Nanny_Set_Index(out: &mut bool, object: &mut Local, index: u32, val: &mut Local) -> bool;
    pub fn Nanny_Get_Bytes(out: &mut Local, object: &mut Local, key: *const u8, len: i32) -> bool;
    pub fn Nanny_Set_Bytes(out: &mut bool, object: &mut Local, key: *const u8, len: i32, val: &mut Local) -> bool;
    pub fn Nan_Get(out: &mut Local, object: &mut Local, key: &mut Local) -> bool;
    pub fn Nan_Set(out: &mut bool, object: &mut Local, key: &mut Local, val: &Local) -> bool;
    pub fn Node_ArrayLength(array: &Local) -> u32;
    pub fn Nan_String_Utf8Length(str: &Local) -> isize;

    pub fn Nan_Value_ToObject(out: &mut Local, value: &Local) -> bool;
    pub fn Nan_Value_ToString(out: &mut Local, value: &mut Local) -> bool;

    pub fn Nan_NewBuffer(out: &mut Local, size: u32) -> bool;
    pub fn Node_Buffer_Data<'a, 'b>(out: &'a mut Buf<'b>, obj: &'a Local);
    pub fn Node_Buffer_Object_HasInstance(obj: &Local) -> bool;
    pub fn Node_Buffer_Value_HasInstance(obj: &Local) -> bool;

    pub fn Nan_Chained(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void), parent_scope: *mut c_void);
    pub fn Nan_Nested(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void), realm: *mut c_void);
    pub fn Nanny_ExecFunctionBody(closure: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void), info: &FunctionCallbackInfo, scope: *mut c_void);
    pub fn Nanny_ExecModuleBody(kernel: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void), exports: &mut Local, scope: *mut c_void);

    pub fn Nanny_FunctionKernel(obj: &Local) -> *mut c_void;
    pub fn Nanny_NewFunction(out: &mut Local, isolate: *mut c_void, callback: *mut c_void, kernel: *mut c_void) -> bool;
    pub fn Nanny_TagOf(val: &Local) -> Tag;
    pub fn Nanny_IsUndefined(val: &Local) -> bool;
    pub fn Nanny_IsNull(val: &Local) -> bool;
    pub fn Nanny_IsInteger(val: &Local) -> bool;
    pub fn Nanny_IsNumber(val: &Local) -> bool;
    pub fn Nanny_IsBoolean(val: &Local) -> bool;
    pub fn Nanny_IsString(val: &Local) -> bool;
    pub fn Nanny_IsObject(val: &Local) -> bool;
    pub fn Nanny_IsArray(val: &Local) -> bool;
    pub fn Nanny_IsFunction(val: &Local) -> bool;
    pub fn Nanny_IsTypeError(val: &Local) -> bool;

    pub fn Nanny_ThrowAny(val: &Local);
    pub fn Nanny_NewTypeError(out: &mut Local, msg: *const u8) -> bool;
    pub fn Nanny_ThrowTypeError(msg: *const u8);
}
