use std;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use neon_runtime;
use neon_runtime::raw;
use neon_runtime::scope::Root;
use types::JsObject;
use handle::Handle;
use object::class::ClassMap;
use result::NeonResult;
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

pub struct Scope<'a, R: Root + 'static> {
    pub metadata: ScopeMetadata,
    pub handle_scope: &'a mut R,
    pub handle_arena: HandleArena<'a>
}

const CHUNK_SIZE: usize = 16;

type Chunk = [raw::Local; CHUNK_SIZE];

pub struct HandleArena<'a> {
    chunks: VecDeque<Box<Chunk>>,
    chunk_allocated: usize,
    phantom: PhantomData<&'a ()>
}

impl<'a> HandleArena<'a> {
    fn new() -> Self {
        let mut chunks = VecDeque::with_capacity(16);

        chunks.push_back(Box::new(unsafe { mem::uninitialized() }));

        HandleArena {
            chunks,
            chunk_allocated: 0,
            phantom: PhantomData
        }
    }

    pub unsafe fn alloc(&mut self) -> *mut raw::Local {
        if self.chunk_allocated >= CHUNK_SIZE {
            let mut chunk: Box<Chunk> = Box::new(mem::uninitialized());
            let p: *mut raw::Local = &mut chunk[0];
            self.chunks.push_back(chunk);
            self.chunk_allocated = 1;
            p
        } else {
            let chunk: &mut Box<Chunk> = self.chunks.back_mut().unwrap();
            let p: *mut raw::Local = &mut chunk[self.chunk_allocated];
            self.chunk_allocated += 1;
            p
        }
    }
/*
    fn alloc(&mut self, init: raw::Local) -> &'a raw::Local {
        let p = if self.chunk_allocated >= CHUNK_SIZE {
            let mut chunk: Box<Chunk> = Box::new(unsafe { mem::uninitialized() });
            chunk[0] = init;
            let p: *const raw::Local = &chunk[0];
            self.chunks.push_back(chunk);
            self.chunk_allocated = 1;
            p
        } else {
            let chunk: &mut Box<Chunk> = self.chunks.back_mut().unwrap();
            chunk[self.chunk_allocated] = init;
            let p: *const raw::Local = &chunk[self.chunk_allocated];
            self.chunk_allocated += 1;
            p
        };

        unsafe { mem::transmute(p) }
    }
*/

}

impl<'a, R: Root + 'static> Scope<'a, R> {
    pub fn with<T, F: for<'b> FnOnce(Scope<'b, R>) -> T>(f: F) -> T {
        let mut handle_scope: R = unsafe { R::allocate() };
        let isolate = Isolate::current();
        unsafe {
            handle_scope.enter(isolate.to_raw());
        }
        let result = {
            let scope = Scope {
                metadata: ScopeMetadata {
                    isolate,
                    active: Cell::new(true)
                },
                handle_scope: &mut handle_scope,
                handle_arena: HandleArena::new()
            };
            f(scope)
        };
        unsafe {
            handle_scope.exit();
        }
        result
    }
}

pub trait ContextInternal<'a>: Sized {
    fn scope_metadata(&self) -> &ScopeMetadata; 
    fn handle_arena(&mut self) -> &mut HandleArena<'a>;

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

    unsafe fn alloc(&mut self) -> *mut raw::Local {
        self.handle_arena().alloc()
    }
}

pub fn initialize_module(exports: Handle<JsObject>, init: fn(ModuleContext) -> NeonResult<()>) {
    ModuleContext::with(exports, |cx| {
        let _ = init(cx);
    });
}
