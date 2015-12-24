use raw::{FunctionCallbackInfo, Isolate, Local};

extern "system" {

    #[link_name = "NeonSys_Call_SetReturn"]
    pub fn SetReturn(info: &FunctionCallbackInfo, value: Local);

    #[link_name = "NeonSys_Call_GetIsolate"]
    pub fn GetIsolate(info: &FunctionCallbackInfo) -> &Isolate;

    #[link_name = "NeonSys_Call_IsConstruct"]
    pub fn IsConstruct(info: &FunctionCallbackInfo) -> bool;

    #[link_name = "NeonSys_Call_This"]
    pub fn This(info: &FunctionCallbackInfo, out: &mut Local);

    #[link_name = "NeonSys_Call_Callee"]
    pub fn Callee(info: &FunctionCallbackInfo, out: &mut Local);

    #[link_name = "NeonSys_Call_Data"]
    pub fn Data(info: &FunctionCallbackInfo, out: &mut Local);

    #[link_name = "NeonSys_Call_Length"]
    pub fn Length(info: &FunctionCallbackInfo) -> i32;

    #[link_name = "NeonSys_Call_Get"]
    pub fn Get(info: &FunctionCallbackInfo, i: i32, out: &mut Local);

}
