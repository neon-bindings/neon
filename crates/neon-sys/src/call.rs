//! Facilities for working with `v8::FunctionCallbackInfo` and getting the current `v8::Isolate`.

use raw::{FunctionCallbackInfo, Isolate, Local};

extern "system" {

    /// Sets the return value of the function call.
    #[link_name = "NeonSys_Call_SetReturn"]
    pub fn set_return(info: &FunctionCallbackInfo, value: Local);

    /// Gets the isolate of the function call.
    #[link_name = "NeonSys_Call_GetIsolate"]
    pub fn get_isolate(info: &FunctionCallbackInfo) -> *mut Isolate;

    /// Gets the current `v8::Isolate`.
    #[link_name = "NeonSys_Call_CurrentIsolate"]
    pub fn current_isolate() -> *mut Isolate;

    /// Indicates if the function call was invoked as a constructor.
    #[link_name = "NeonSys_Call_IsConstruct"]
    pub fn is_construct(info: &FunctionCallbackInfo) -> bool;

    /// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the object
    /// the function is bound to.
    #[link_name = "NeonSys_Call_This"]
    pub fn this(info: &FunctionCallbackInfo, out: &mut Local);

    /// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the
    /// currently executing function.
    #[link_name = "NeonSys_Call_Callee"]
    pub fn callee(info: &FunctionCallbackInfo, out: &mut Local);

    /// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the
    /// `v8::FunctionCallbackInfo` `Data`.
    #[link_name = "NeonSys_Call_Data"]
    pub fn data(info: &FunctionCallbackInfo, out: &mut Local);

    /// Gets the number of arguments passed to the function.
    #[link_name = "NeonSys_Call_Length"]
    pub fn len(info: &FunctionCallbackInfo) -> i32;

    /// Mutates the `out` argument provided to refer to the `v8::Local` handle value of the `i`th
    /// argument passed to the function.
    #[link_name = "NeonSys_Call_Get"]
    pub fn get(info: &FunctionCallbackInfo, i: i32, out: &mut Local);

}
