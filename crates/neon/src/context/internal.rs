use std::{cell::RefCell, ffi::c_void, mem::MaybeUninit};

use crate::{
    context::{Cx, ModuleContext},
    handle::Handle,
    result::NeonResult,
    sys::{self, raw},
    types::{private::ValueInternal, JsObject},
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
    pub(crate) static IS_RUNNING: RefCell<bool> = const { RefCell::new(false) };
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

pub trait ContextInternal<'cx>: Sized {
    fn cx(&self) -> &Cx<'cx>;
    fn cx_mut(&mut self) -> &mut Cx<'cx>;
    fn env(&self) -> Env {
        self.cx().env
    }
}

fn default_main(mut cx: ModuleContext) -> NeonResult<()> {
    #[cfg(all(feature = "napi-6", feature = "tokio-rt-multi-thread"))]
    crate::executor::tokio::init(&mut cx)?;
    crate::registered().export(&mut cx)
}

fn init(cx: ModuleContext) -> NeonResult<()> {
    if crate::macro_internal::MAIN.len() > 1 {
        panic!("The `neon::main` macro must only be used once");
    }

    if let Some(main) = crate::macro_internal::MAIN.first() {
        main(cx)
    } else {
        default_main(cx)
    }
}

#[no_mangle]
unsafe extern "C" fn napi_register_module_v1(env: *mut c_void, m: *mut c_void) -> *mut c_void {
    let env = env.cast();

    sys::setup(env);

    IS_RUNNING.with(|v| {
        *v.borrow_mut() = true;
    });

    let env = Env(env);
    let exports = Handle::new_internal(JsObject::from_local(env, m.cast()));
    let _ = ModuleContext::with(env, exports, init);

    m
}
