use std::ptr;

use nodejs_sys as napi;

pub type Local = napi::napi_value;

pub type FunctionCallbackInfo = napi::napi_callback_info;

pub type Env = napi::napi_env;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HandleScope {
    pub word: napi::napi_handle_scope
}

impl HandleScope {
    pub fn new() -> Self { Self { word: ptr::null_mut() } }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EscapableHandleScope {
    pub word: napi::napi_escapable_handle_scope
}

impl EscapableHandleScope {
    pub fn new() -> Self { Self { word: ptr::null_mut() } }
}

#[derive(Clone, Copy)]
pub struct InheritedHandleScope;
