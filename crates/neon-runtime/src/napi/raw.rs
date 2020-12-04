use std::ptr;

use crate::napi::bindings as napi;

pub type Local = napi::Value;

pub type FunctionCallbackInfo = napi::CallbackInfo;

pub type Env = napi::Env;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HandleScope {
    pub word: napi::HandleScope
}

impl HandleScope {
    pub fn new() -> Self { Self { word: ptr::null_mut() } }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EscapableHandleScope {
    pub word: napi::EscapableHandleScope
}

impl EscapableHandleScope {
    pub fn new() -> Self { Self { word: ptr::null_mut() } }
}

#[derive(Clone, Copy)]
pub struct InheritedHandleScope;
