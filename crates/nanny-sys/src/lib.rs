pub mod raw;

use std::os::raw::c_void;
use raw::{FunctionCallbackInfo, Local};

extern "system" {
    pub fn Nan_FunctionCallbackInfo_SetReturnValue(info: &mut FunctionCallbackInfo, value: Local);
    pub fn Nan_Export(target: &mut Local, name: *const u8, f: extern fn(&mut FunctionCallbackInfo));
    pub fn Nan_NewObject(out: &mut Local);
    /*
    pub fn Nan_NewString(value: *const u8) -> MaybeLocalString;
    pub fn Nan_NewStringN(value: *const u8, length: i32) -> MaybeLocalString;
     */
    pub fn Nan_NewInteger(out: &mut Local, x: i32);
    pub fn Nan_NewNumber(out: &mut Local, v: f64);
    pub fn Nan_NewArray(out: &mut Local, length: u32);
    pub fn Nan_ArraySet(array: &mut Local, index: u32, value: Local) -> bool;

    /*
    pub fn Nan_MaybeLocalString_ToOption(maybe: &MaybeLocalString, out: &mut LocalString) -> bool;
    pub fn Nan_MaybeLocalString_IsEmpty(maybe: &MaybeLocalString) -> bool;
     */

    //pub fn Nan_MaybeLocalString_ToLocal(maybe: &MaybeLocalString, out: &mut LocalString) -> bool;
    //pub fn Nan_MaybeLocalString_ToLocal(maybe: &mut MaybeLocalString, &mut ) -> LocalString;

    pub fn Nan_Scoped(out: *mut c_void, closure: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void));
    pub fn Nan_EscapeScoped(out: *mut c_void, closure: *mut c_void, callback: extern fn(*mut c_void, *mut c_void, *mut c_void));
}
