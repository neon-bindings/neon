use std::ptr;

use super::bindings as napi;

pub type Local = napi::Value;

pub type FunctionCallbackInfo = napi::CallbackInfo;

pub type Env = napi::Env;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HandleScope {
    pub word: napi::HandleScope,
}

impl HandleScope {
    pub fn new() -> Self {
        Self {
            word: ptr::null_mut(),
        }
    }
}

impl Default for HandleScope {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EscapableHandleScope {
    pub word: napi::EscapableHandleScope,
}

impl EscapableHandleScope {
    pub fn new() -> Self {
        Self {
            word: ptr::null_mut(),
        }
    }
}

impl Default for EscapableHandleScope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct InheritedHandleScope;
