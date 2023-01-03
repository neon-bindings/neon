use std::mem::MaybeUninit;

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub(crate) struct HandleScope {
    env: Env,
    scope: napi::HandleScope,
}

impl HandleScope {
    pub(crate) unsafe fn new(env: Env) -> Self {
        let mut scope = MaybeUninit::uninit();

        assert_eq!(
            napi::open_handle_scope(env, scope.as_mut_ptr()),
            napi::Status::Ok,
        );

        Self {
            env,
            scope: scope.assume_init(),
        }
    }
}

impl Drop for HandleScope {
    fn drop(&mut self) {
        unsafe {
            let _status = napi::close_handle_scope(self.env, self.scope);

            debug_assert_eq!(_status, napi::Status::Ok,);
        }
    }
}

pub(crate) struct EscapableHandleScope {
    env: Env,
    scope: napi::EscapableHandleScope,
}

impl EscapableHandleScope {
    pub(crate) unsafe fn new(env: Env) -> Self {
        let mut scope = MaybeUninit::uninit();

        assert_eq!(
            napi::open_escapable_handle_scope(env, scope.as_mut_ptr()),
            napi::Status::Ok,
        );

        Self {
            env,
            scope: scope.assume_init(),
        }
    }

    pub(crate) unsafe fn escape(&self, value: napi::Value) -> napi::Value {
        let mut escapee = MaybeUninit::uninit();

        assert_eq!(
            napi::escape_handle(self.env, self.scope, value, escapee.as_mut_ptr()),
            napi::Status::Ok,
        );

        escapee.assume_init()
    }
}

impl Drop for EscapableHandleScope {
    fn drop(&mut self) {
        unsafe {
            let _status = napi::close_escapable_handle_scope(self.env, self.scope);

            debug_assert_eq!(_status, napi::Status::Ok,);
        }
    }
}

pub unsafe fn get_global(env: Env, out: &mut Local) {
    assert_eq!(super::get_global(env, out as *mut _), napi::Status::Ok);
}
