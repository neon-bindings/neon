
use std::mem::MaybeUninit;

use crate::raw::{Env, HandleScope, EscapableHandleScope, InheritedHandleScope, Local};
use crate::napi::bindings as napi;

// TODO: This leaves a lot of room for UB; we can have a cleaner
// implementation for N-API.
pub trait Root {
    unsafe fn allocate() -> Self;
    unsafe fn enter(&mut self, env: Env);
    unsafe fn exit(&mut self, env: Env);
}

impl Root for HandleScope {
    unsafe fn allocate() -> Self { HandleScope::new() }
    unsafe fn enter(&mut self, env: Env) {
        let mut scope = MaybeUninit::uninit();
        let status = napi::open_handle_scope(env, scope.as_mut_ptr());

        assert_eq!(status, napi::Status::Ok);

        self.word = scope.assume_init();
    }
    unsafe fn exit(&mut self, env: Env) {
        let status = napi::close_handle_scope(env, self.word);

        assert_eq!(status, napi::Status::Ok);
    }
}

impl Root for EscapableHandleScope {
    unsafe fn allocate() -> Self { EscapableHandleScope::new() }
    unsafe fn enter(&mut self, env: Env) {
        let mut scope = MaybeUninit::uninit();
        let status = napi::open_escapable_handle_scope(env, scope.as_mut_ptr());

        assert_eq!(status, napi::Status::Ok);

        self.word = scope.assume_init();
    }
    unsafe fn exit(&mut self, env: Env) {
        let status = napi::close_escapable_handle_scope(env, self.word);

        assert_eq!(status, napi::Status::Ok);
    }
}

impl Root for InheritedHandleScope {
    unsafe fn allocate() -> Self { InheritedHandleScope }
    unsafe fn enter(&mut self, _: Env) { }
    unsafe fn exit(&mut self, _: Env) { }
}

pub unsafe extern "C" fn escape(env: Env, out: &mut Local, scope: *mut EscapableHandleScope, value: Local) {
    let status = napi::escape_handle(env, (*scope).word, value, out as *mut _);

    assert_eq!(status, napi::Status::Ok);
}

pub unsafe extern "C" fn get_global(env: Env, out: &mut Local) {
    assert_eq!(crate::napi::bindings::get_global(env, out as *mut _), napi::Status::Ok);
}
