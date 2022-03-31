use super::ModuleContext;
use crate::handle::Handle;
use crate::result::NeonResult;
use crate::types::{JsObject, JsValue};
use neon_runtime;
use neon_runtime::raw;
use neon_runtime::scope::Root;
use std::cell::{Cell, RefCell};
use std::mem::MaybeUninit;

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

    unsafe fn try_catch<T, F>(self, f: F) -> Result<T, raw::Local>
    where
        F: FnOnce() -> Result<T, crate::result::Throw>,
    {
        let result = f();
        let mut local: MaybeUninit<raw::Local> = MaybeUninit::zeroed();

        if neon_runtime::error::catch_error(self.to_raw(), local.as_mut_ptr()) {
            Err(local.assume_init())
        } else if let Ok(result) = result {
            Ok(result)
        } else {
            panic!("try_catch: unexpected Err(Throw) when VM is not in a throwing state");
        }
    }
}

pub struct ScopeMetadata {
    env: Env,
    active: Cell<bool>,
}

pub struct Scope<'a, R: Root + 'static> {
    pub metadata: ScopeMetadata,
    pub handle_scope: &'a mut R,
}

impl<'a, R: Root + 'static> Scope<'a, R> {
    pub fn with<T, F: for<'b> FnOnce(Scope<'b, R>) -> T>(env: Env, f: F) -> T {
        let mut handle_scope: R = unsafe { R::allocate() };
        unsafe {
            handle_scope.enter(env.to_raw());
        }
        let result = {
            let scope = Scope {
                metadata: ScopeMetadata {
                    env,
                    active: Cell::new(true),
                },
                handle_scope: &mut handle_scope,
            };
            f(scope)
        };
        unsafe {
            handle_scope.exit(env.to_raw());
        }
        result
    }
}

pub trait ContextInternal<'a>: Sized {
    fn scope_metadata(&self) -> &ScopeMetadata;

    fn env(&self) -> Env {
        self.scope_metadata().env
    }

    fn is_active(&self) -> bool {
        self.scope_metadata().active.get()
    }

    fn check_active(&self) {
        if !self.is_active() {
            panic!("execution context is inactive");
        }
    }

    fn activate(&self) {
        self.scope_metadata().active.set(true);
    }
    fn deactivate(&self) {
        self.scope_metadata().active.set(false);
    }

    fn try_catch_internal<T, F>(&mut self, f: F) -> Result<T, Handle<'a, JsValue>>
    where
        F: FnOnce(&mut Self) -> NeonResult<T>,
    {
        unsafe {
            self.env()
                .try_catch(move || f(self))
                .map_err(JsValue::new_internal)
        }
    }
}

pub fn initialize_module(
    env: raw::Env,
    exports: Handle<JsObject>,
    init: fn(ModuleContext) -> NeonResult<()>,
) {
    unsafe {
        neon_runtime::setup(env);
    }

    IS_RUNNING.with(|v| {
        *v.borrow_mut() = true;
    });

    ModuleContext::with(Env(env), exports, |cx| {
        let _ = init(cx);
    });
}
