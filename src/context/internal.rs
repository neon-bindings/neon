use std;
use std::cell::Cell;
use std::mem;
use std::os::raw::c_void;
use neon_runtime;
use neon_runtime::raw;
use typed_arena::Arena;
use types::{JsObject, Managed};
use object::class::ClassMap;
use result::{NeonResult, Throw};
use super::ModuleContext;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Isolate(*mut raw::Isolate);

extern "C" fn drop_class_map(map: Box<ClassMap>) {
    std::mem::drop(map);
}

impl Isolate {
    pub(crate) fn to_raw(self) -> *mut raw::Isolate {
        let Isolate(ptr) = self;
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

    pub(crate) fn current() -> Isolate {
        unsafe {
            std::mem::transmute(neon_runtime::call::current_isolate())
        }
    }
}

pub struct ScopeMetadata {
    isolate: Isolate,
    active: Cell<bool>
}

pub struct Scope<'a> {
    pub metadata: ScopeMetadata,
    pub handles: &'a PersistentArena,
}

// FIXME: rename to HandleArena
pub struct PersistentArena {
    handles: Arena<raw::Persistent>,
}

impl PersistentArena {
    fn new() -> Self {
        PersistentArena {
            handles: Arena::with_capacity(16),
        }
    }

    pub fn alloc(&self) -> &raw::Persistent {
        unsafe {
            let ptr = self.handles.alloc_uninitialized(1);
            let ptr = ptr as *mut raw::Persistent;
            raw::Persistent::placement_new(ptr);
            let ptr = ptr as *const raw::Persistent;
            mem::transmute(ptr)
        }
    }

    pub(crate) fn clone<'a, 'b>(&'a self, isolate: *mut raw::Isolate, handle: &'b raw::Persistent) -> &'a raw::Persistent {
        let new_ptr = self.alloc();
        unsafe {
            neon_runtime::scope::clone_persistent(isolate, new_ptr, handle);
        }
        new_ptr
    }
}

impl<'a> Scope<'a> {
    pub fn with<T, F: for<'b> FnOnce(Scope<'b>) -> T>(f: F) -> T {
        let handles = PersistentArena::new();
        let isolate = Isolate::current();
        let scope = Scope {
            metadata: ScopeMetadata {
                isolate,
                active: Cell::new(true)
            },
            handles: &handles,
        };
        f(scope)
    }
}

pub trait ContextInternal<'a>: Sized {
    fn scope_metadata(&self) -> &ScopeMetadata; 
    fn handles(&self) -> &'a PersistentArena;

    fn isolate(&self) -> Isolate {
        self.scope_metadata().isolate
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

    fn new<T: Managed, F: FnOnce(&raw::Persistent, *mut raw::Isolate) -> bool>(&mut self, init: F) -> NeonResult<&'a T> {
        self.new_opt(init).ok_or(Throw)
    }

    fn new_infallible<T: Managed, F: FnOnce(&raw::Persistent, *mut raw::Isolate)>(&mut self, init: F) -> &'a T {
        let isolate = { self.isolate().to_raw() };
        let h = self.handles().alloc();
        init(h, isolate);
        T::from_raw(h)
    }

    fn new_opt<T: Managed, F: FnOnce(&raw::Persistent, *mut raw::Isolate) -> bool>(&mut self, init: F) -> Option<&'a T> {
        let isolate = { self.isolate().to_raw() };
        let h = self.handles().alloc();
        if init(h, isolate) {
            Some(T::from_raw(h))
        } else {
            None
        }
    }
}

pub fn initialize_module(exports: raw::Local, init: fn(ModuleContext) -> NeonResult<()>) {
    // FIXME: this leaks if the module initialization panics
    let persistent = raw::Persistent::from_local(exports);
    ModuleContext::with(JsObject::from_raw(&persistent), |cx| {
        let _ = init(cx);
    });
}
