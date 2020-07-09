use std;
use std::boxed::Box;
use std::cell::Cell;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use neon_runtime;
use neon_runtime::raw;
use neon_runtime::scope::Root;
use types::{JsObject, JsValue};
use handle::{Handle, Managed};
use object::class::ClassMap;
use result::{JsResult, NeonResult};
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

    fn try_catch_internal<'b: 'a, F>(&mut self, f: F) -> Result<Handle<'a, JsValue>, Handle<'a, JsValue>>
        where F: FnOnce(&mut Self) -> JsResult<'b, JsValue>
    {
        let p = Box::into_raw(Box::new(f)) as *mut c_void;
        let mut local: MaybeUninit<raw::Local> = MaybeUninit::zeroed();
        let threw = unsafe { neon_runtime::try_catch::with(try_catch_glue::<Self, F>, self as *mut Self as *mut c_void, p, &mut *local.as_mut_ptr()) };
        let local = unsafe { local.assume_init() };
        if threw {
            Err(JsValue::new_internal(local))
        } else {
            Ok(JsValue::new_internal(local))
        }
    }
}

extern "C" fn try_catch_glue<'a, 'b: 'a, C: ContextInternal<'a>, F>(p: *mut c_void, cx: *mut c_void, ok: *mut bool, ok_value: *mut raw::Local)
    where C: ContextInternal<'a>,
          F: FnOnce(&mut C) -> JsResult<'b, JsValue>
{
    let f: F = *unsafe { Box::from_raw(p as *mut F) };

    // FIXME: convert_panics(f)
    match f(unsafe { std::mem::transmute(cx) }) {
        Ok(result) => unsafe {
            *ok = true;
            *ok_value = result.to_raw();
        }
        Err(_) => unsafe {
            *ok = false;
        }
    }
}

#[cfg(feature = "legacy-runtime")]
pub fn initialize_module(exports: Handle<JsObject>, init: fn(ModuleContext) -> NeonResult<()>) {
    let env = Env::current();

    ModuleContext::with(env, exports, |cx| {
        let _ = init(cx);
    });
}

#[cfg(feature = "napi-runtime")]
pub fn initialize_module(env: raw::Env, exports: Handle<JsObject>, init: fn(ModuleContext) -> NeonResult<()>) {
    ModuleContext::with(Env(env), exports, |cx| {
        let _ = init(cx);
    });
}
