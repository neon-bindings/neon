use std::mem;
use std::os::raw::c_void;
use std::marker::PhantomData;
use std::cell::{RefCell, UnsafeCell};
use nanny_sys::raw;
use nanny_sys::{Nan_Nested, Nan_Chained, Nan_EscapableHandleScope_Escape};
use internal::mem::{Handle, HandleInternal};
use internal::value::Tagged;
use internal::vm::Isolate;

pub trait ScopeInternal: Sized {
    fn isolate(&self) -> *mut Isolate;
    fn active_cell(&self) -> &RefCell<bool>;
}

pub trait Scope<'a>: ScopeInternal {
    fn nested<T, F: for<'inner> FnOnce(&mut NestedScope<'inner>) -> T>(&self, f: F) -> T;
    fn chained<T, F: for<'inner> FnOnce(&mut ChainedScope<'inner, 'a>) -> T>(&self, f: F) -> T;
}

fn ensure_active<T: ScopeInternal>(scope: &T) {
    if !*scope.active_cell().borrow() {
        panic!("illegal attempt to nest in inactive scope");
    }
}

pub struct RootScope<'a> {
    isolate: *mut Isolate,
    active: RefCell<bool>,
    phantom: PhantomData<&'a ()>
}

pub struct NestedScope<'a> {
    isolate: *mut Isolate,
    active: RefCell<bool>,
    phantom: PhantomData<&'a ()>
}

pub struct ChainedScope<'a, 'outer> {
    isolate: *mut Isolate,
    active: RefCell<bool>,
    v8: *mut raw::EscapableHandleScope,
    parent: PhantomData<&'outer ()>,
    phantom: PhantomData<&'a ()>
}

impl<'a, 'outer> ChainedScope<'a, 'outer> {
    pub fn escape<T: Tagged>(&self, local: Handle<'a, T>) -> Handle<'outer, T> {
        let result: UnsafeCell<Handle<'outer, T>> = UnsafeCell::new(Handle::new(unsafe { mem::zeroed() }));
        unsafe {
            Nan_EscapableHandleScope_Escape((*result.get()).to_raw_mut_ref(), self.v8, local.to_raw());
            result.into_inner()
        }
    }
}

pub trait RootScopeInternal<'a> {
    fn new(isolate: *mut Isolate) -> RootScope<'a>;
}

impl<'a> RootScopeInternal<'a> for RootScope<'a> {
    fn new(isolate: *mut Isolate) -> RootScope<'a> {
        RootScope {
            isolate: isolate,
            active: RefCell::new(true),
            phantom: PhantomData
        }
    }
}

impl<'a> Scope<'a> for RootScope<'a> {
    fn nested<T, F: for<'inner> FnOnce(&mut NestedScope<'inner>) -> T>(&self, f: F) -> T {
        nest(self, f)
    }

    fn chained<T, F: for<'inner> FnOnce(&mut ChainedScope<'inner, 'a>) -> T>(&self, f: F) -> T {
        chain(self, f)
    }
}

extern "C" fn chained_callback<'a, T, P, F>(out: &mut Box<Option<T>>,
                                            parent: &P,
                                            v8: *mut raw::EscapableHandleScope,
                                            f: Box<F>)
    where P: Scope<'a>,
          F: for<'inner> FnOnce(&mut ChainedScope<'inner, 'a>) -> T
{
    let mut chained = ChainedScope {
        isolate: parent.isolate(),
        active: RefCell::new(true),
        v8: v8,
        parent: PhantomData,
        phantom: PhantomData
    };
    let result = f(&mut chained);
    **out = Some(result);
}

impl<'a> ScopeInternal for RootScope<'a> {
    fn isolate(&self) -> *mut Isolate { self.isolate }

    fn active_cell(&self) -> &RefCell<bool> {
        &self.active
    }
}

fn chain<'a, T, S, F>(outer: &S, f: F) -> T
    where S: Scope<'a>,
          F: for<'inner> FnOnce(&mut ChainedScope<'inner, 'a>) -> T
{
    ensure_active(outer);
    let closure: Box<F> = Box::new(f);
    let callback: extern "C" fn(&mut Box<Option<T>>, &S, *mut raw::EscapableHandleScope, Box<F>) = chained_callback::<'a, T, S, F>;
    let mut result: Box<Option<T>> = Box::new(None);
    {
        let out: &mut Box<Option<T>> = &mut result;
        { *outer.active_cell().borrow_mut() = false; }
        unsafe {
            let out: *mut c_void = mem::transmute(out);
            let closure: *mut c_void = mem::transmute(closure);
            let callback: extern "C" fn(&mut c_void, *mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
            let this: *mut c_void = mem::transmute(outer);
            Nan_Chained(out, closure, callback, this);
        }
        { *outer.active_cell().borrow_mut() = true; }
    }
    result.unwrap()
}

fn nest<'me, T, S, F>(outer: &'me S, f: F) -> T
    where S: ScopeInternal,
          F: for<'nested> FnOnce(&mut NestedScope<'nested>) -> T
{
    ensure_active(outer);
    let closure: Box<F> = Box::new(f);
    let callback: extern "C" fn(&mut Box<Option<T>>, *mut Isolate, Box<F>) = nested_callback::<T, F>;
    let mut result: Box<Option<T>> = Box::new(None);
    {
        let out: &mut Box<Option<T>> = &mut result;
        { *outer.active_cell().borrow_mut() = false; }
        unsafe {
            let out: *mut c_void = mem::transmute(out);
            let closure: *mut c_void = mem::transmute(closure);
            let callback: extern "C" fn(&mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
            let isolate: *mut c_void = mem::transmute(outer.isolate());
            Nan_Nested(out, closure, callback, isolate);
        }
        { *outer.active_cell().borrow_mut() = true; }
    }
    result.unwrap()
}

extern "C" fn nested_callback<T, F>(out: &mut Box<Option<T>>,
                                    isolate: *mut Isolate,
                                    f: Box<F>)
    where F: for<'nested> FnOnce(&mut NestedScope<'nested>) -> T
{
    let mut nested = NestedScope {
        isolate: isolate,
        active: RefCell::new(true),
        phantom: PhantomData
    };
    let result = f(&mut nested);
    **out = Some(result);
}

impl<'a> Scope<'a> for NestedScope<'a> {
    fn nested<T, F: for<'inner> FnOnce(&mut NestedScope<'inner>) -> T>(&self, f: F) -> T {
        nest(self, f)
    }

    fn chained<T, F: for<'inner> FnOnce(&mut ChainedScope<'inner, 'a>) -> T>(&self, f: F) -> T {
        chain(self, f)
    }
}

impl<'a> ScopeInternal for NestedScope<'a> {
    fn isolate(&self) -> *mut Isolate { self.isolate }

    fn active_cell(&self) -> &RefCell<bool> {
        &self.active
    }
}

impl<'a, 'outer> Scope<'a> for ChainedScope<'a, 'outer> {
    fn nested<T, F: for<'inner> FnOnce(&mut NestedScope<'inner>) -> T>(&self, f: F) -> T {
        nest(self, f)
    }

    fn chained<T, F: for<'inner> FnOnce(&mut ChainedScope<'inner, 'a>) -> T>(&self, f: F) -> T {
        chain(self, f)
    }
}

impl<'a, 'outer> ScopeInternal for ChainedScope<'a, 'outer> {
    fn isolate(&self) -> *mut Isolate { self.isolate }

    fn active_cell(&self) -> &RefCell<bool> {
        &self.active
    }
}
