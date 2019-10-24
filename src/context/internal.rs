use std;
use std::cell::Cell;
use std::os::raw::c_void;
use neon_runtime;
use neon_runtime::raw;
use neon_runtime::scope::Root;
use types::JsObject;
use handle::Handle;
use object::class::ClassMap;
use result::NeonResult;
use super::ModuleContext;

#[cfg(feature = "legacy-runtime")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Env(raw::Isolate);

#[cfg(feature = "napi-runtime")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Env(raw::Env);

extern "C" fn drop_class_map(map: Box<ClassMap>) {
    std::mem::drop(map);
}

impl Env {
    #[cfg(feature = "legacy-runtime")]
    pub(crate) fn to_raw(self) -> raw::Isolate {
        let Self(ptr) = self;
        ptr
    }

    #[cfg(feature = "napi-runtime")]
    pub(crate) fn to_raw(self) -> raw::Env {
        let Self(ptr) = self;
        ptr
    }

    pub(crate) fn class_map(&mut self) -> &mut ClassMap {
        let mut ptr: *mut c_void = unsafe { neon_runtime::class::get_class_map(self.to_raw()) };
        if ptr.is_null() {
            let b: Box<ClassMap> = Box::new(ClassMap::new());
            let raw = Box::into_raw(b);
            ptr = unsafe { std::mem::transmute(raw) };
            let free_map: *mut c_void = unsafe { std::mem::transmute(drop_class_map as usize) };
            unsafe {
                neon_runtime::class::set_class_map(self.to_raw(), ptr, free_map);
            }
        }
        unsafe { std::mem::transmute(ptr) }
    }

    #[cfg(feature = "napi-runtime")]
    pub(crate) fn current() -> Env {
        panic!("Context::current() will not implemented with n-api")
    }

    #[cfg(feature = "legacy-runtime")]
    pub(crate) fn current() -> Env {
        unsafe {
            std::mem::transmute(neon_runtime::call::current_isolate())
        }
    }
}

pub struct ScopeMetadata {
    env: Env,
    active: Cell<bool>
}

pub struct Scope<'a, R: Root + 'static> {
    pub metadata: ScopeMetadata,
    pub handle_scope: &'a mut R
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
                    active: Cell::new(true)
                },
                handle_scope: &mut handle_scope
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

    fn activate(&self) { self.scope_metadata().active.set(true); }
    fn deactivate(&self) { self.scope_metadata().active.set(false); }
}

#[cfg(feature = "legacy-runtime")]
pub fn initialize_module(exports: Handle<JsObject>, init: fn(ModuleContext) -> NeonResult<()>) {
    let env = Env::current();

    ModuleContext::with(env, exports, |cx| {
        let _ = init(cx);
    });
}

#[cfg(feature = "napi-runtime")]
pub fn initialize_module(env: Env, exports: Handle<JsObject>, init: fn(ModuleContext) -> NeonResult<()>) {
    ModuleContext::with(env, exports, |cx| {
        let _ = init(cx);
    });
}
