use std::{cell::RefCell, ffi::c_void, mem::MaybeUninit};

use crate::{
    context::ModuleContext,
    handle::{Handle, Managed},
    result::NeonResult,
    sys::{self, raw},
    types::JsObject,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Env(raw::Env);

impl From<raw::Env> for Env {
    fn from(env: raw::Env) -> Self {
        Self(env)
    }
}

thread_local! {
    #[allow(unused)]
    pub(crate) static IS_RUNNING: RefCell<bool> = RefCell::new(false);
}

impl Env {
    pub(crate) fn to_raw(self) -> raw::Env {
        let Self(ptr) = self;
        ptr
    }

    pub(super) unsafe fn try_catch<T, F>(self, f: F) -> Result<T, raw::Local>
    where
        F: FnOnce() -> Result<T, crate::result::Throw>,
    {
        let result = f();
        let mut local: MaybeUninit<raw::Local> = MaybeUninit::zeroed();

        if sys::error::catch_error(self.to_raw(), local.as_mut_ptr()) {
            Err(local.assume_init())
        } else if let Ok(result) = result {
            Ok(result)
        } else {
            panic!("try_catch: unexpected Err(Throw) when VM is not in a throwing state");
        }
    }
}

pub trait ContextInternal<'a>: Sized {
    fn env(&self) -> Env;
}

pub unsafe fn initialize_module(
    env: *mut c_void,
    exports: *mut c_void,
    init: fn(ModuleContext) -> NeonResult<()>,
) {
    let env = env.cast();

    sys::setup(env);

    IS_RUNNING.with(|v| {
        *v.borrow_mut() = true;
    });

    let env = Env(env);
    let exports = Handle::new_internal(JsObject::from_raw(env, exports.cast()));

    ModuleContext::with(env, exports, |cx| {
        let _ = init(cx);
    });
}
