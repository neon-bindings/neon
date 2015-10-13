pub mod raw;

use std::os::raw::c_void;
use raw::{FunctionCallbackInfo, EscapableHandleScope, Isolate, Local};

extern "system" {
    pub fn Nan_FunctionCallbackInfo_SetReturnValue(info: &mut FunctionCallbackInfo, value: Local);
    pub fn Nan_FunctionCallbackInfo_GetIsolate(info: &FunctionCallbackInfo) -> &Isolate;
    pub fn Nan_FunctionCallbackInfo_IsConstructCall(info: &FunctionCallbackInfo) -> bool;
    pub fn Nan_FunctionCallbackInfo_This(info: &FunctionCallbackInfo, out: &mut Local);
    pub fn Nan_FunctionCallbackInfo_Length(info: &FunctionCallbackInfo) -> i32;
    pub fn Nan_FunctionCallbackInfo_Get(info: &FunctionCallbackInfo, i: i32, out: &mut Local);

    pub fn Nan_EscapableHandleScope_Escape(out: &mut Local, scope: *mut EscapableHandleScope, value: Local);

    pub fn Nan_Export(target: &mut Local, name: *const u8, f: extern fn(&mut FunctionCallbackInfo));
    pub fn Nan_NewObject(out: &mut Local);
    /*
    pub fn Nan_NewString(value: *const u8) -> MaybeLocalString;
    pub fn Nan_NewStringN(value: *const u8, length: i32) -> MaybeLocalString;
     */
    pub fn Nan_NewUndefined(out: &mut Local);
    pub fn Nan_NewNull(out: &mut Local);
    pub fn Nan_NewBoolean(out: &mut Local, b: bool);
    pub fn Nan_NewInteger(out: &mut Local, isolate: *mut Isolate, x: i32);
    pub fn Nan_NewNumber(out: &mut Local, isolate: *mut Isolate, v: f64);
    pub fn Nan_NewArray(out: &mut Local, isolate: *mut Isolate, length: u32);
    pub fn Nan_ArraySet(array: &mut Local, index: u32, value: Local) -> bool;

    /*
    pub fn Nan_MaybeLocalString_ToOption(maybe: &MaybeLocalString, out: &mut LocalString) -> bool;
    pub fn Nan_MaybeLocalString_IsEmpty(maybe: &MaybeLocalString) -> bool;
     */

    //pub fn Nan_MaybeLocalString_ToLocal(maybe: &MaybeLocalString, out: &mut LocalString) -> bool;
    //pub fn Nan_MaybeLocalString_ToLocal(maybe: &mut MaybeLocalString, &mut ) -> LocalString;

    //pub fn Nan_Scoped(out: *mut c_void, closure: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void));
    //pub fn Nan_EscapeScoped(out: *mut c_void, closure: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void));

    pub fn Nan_Chained(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void), parent_scope: *mut c_void);
    pub fn Nan_Nested(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void), realm: *mut c_void);
    pub fn Nan_Root(out: *mut c_void, closure: *mut c_void, callback: extern fn(&mut c_void, *mut c_void, *mut c_void), isolate: *mut c_void);
}
