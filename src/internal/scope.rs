use std::fmt::Debug;
use std::mem;
use std::os::raw::c_void;
use std::marker::PhantomData;
use std::cell::{RefCell, UnsafeCell};
use nanny_sys::raw;
use nanny_sys::{Nan_Nested, Nan_Chained, Nan_EscapableHandleScope_Escape};
use internal::mem::{Handle, HandleInternal};
use internal::value::Value;
use vm::Realm;

pub trait Scope<'root>: Sized {
    fn realm(&self) -> &'root Realm;
    fn nested<'outer, T: Debug, F: FnOnce(&NestedScope<'root>) -> T>(&'outer self, f: F) -> T;
    fn chained<'outer, T: Debug, F: FnOnce(&ChainedScope<'root, 'outer>) -> T>(&'outer self, f: F) -> T;

    // FIXME: define this in a private subtrait?
    fn active(&self) -> bool;
}

trait ScopeInternal<'root>: Scope<'root> {
    fn active_cell(&self) -> &RefCell<bool>;
}

pub struct RootScope<'root> {
    realm: &'root Realm,
    active: RefCell<bool>
}

pub struct NestedScope<'root> {
    realm: &'root Realm,
    active: RefCell<bool>
}

pub struct ChainedScope<'root, 'parent> {
    realm: &'root Realm,
    active: RefCell<bool>,
    v8: *mut raw::EscapableHandleScope,
    parent: PhantomData<&'parent ()>
}

impl<'root, 'parent> ChainedScope<'root, 'parent> {
    pub fn escape<'me, T: Clone + Value>(&'me self, local: Handle<'me, T>) -> Handle<'parent, T> {
        let result: UnsafeCell<Handle<'parent, T>> = UnsafeCell::new(Handle::new(unsafe { mem::zeroed() }));
        unsafe {
            Nan_EscapableHandleScope_Escape((*result.get()).to_raw_mut_ref(), self.v8, local.to_raw());
            result.into_inner()
        }
    }
}

pub trait RootScopeInternal<'root> {
    fn new(realm: &'root Realm, active: RefCell<bool>) -> RootScope<'root>;
}

impl<'root> RootScopeInternal<'root> for RootScope<'root> {
    fn new(realm: &'root Realm, active: RefCell<bool>) -> RootScope<'root> {
        RootScope {
            realm: realm,
            active: active
        }
    }
}

impl<'root> Scope<'root> for RootScope<'root> {
    fn realm(&self) -> &'root Realm { self.realm }

    fn active(&self) -> bool {
        *self.active.borrow()
    }

    fn nested<'me, T: Debug, F: FnOnce(&NestedScope<'root>) -> T>(&'me self, f: F) -> T {
        nest(self, f)
    }

    fn chained<'me, T: Debug, F: FnOnce(&ChainedScope<'root, 'me>) -> T>(&'me self, f: F) -> T {
        chain(self, f)
    }
}

extern "C" fn chained_callback<'root, 'parent, T, P, F>(out: &mut Box<Option<T>>,
                                                        parent: &'parent P,
                                                        v8: *mut raw::EscapableHandleScope,
                                                        f: Box<F>)
    where T: Debug,
          P: Scope<'root>,
          F: FnOnce(&ChainedScope<'root, 'parent>) -> T
{
    let chained = ChainedScope {
        realm: parent.realm(),
        active: RefCell::new(true),
        v8: v8,
        parent: PhantomData
    };
    let result = f(&chained);
    **out = Some(result);
}

impl<'root> ScopeInternal<'root> for RootScope<'root> {
    fn active_cell(&self) -> &RefCell<bool> {
        &self.active
    }
}

fn chain<'root, 'me, T, S, F>(outer: &'me S, f: F) -> T
    where T: Debug,
          S: ScopeInternal<'root>,
          F: FnOnce(&ChainedScope<'root, 'me>) -> T
{
    let closure: Box<F> = Box::new(f);
    let callback: extern "C" fn(&mut Box<Option<T>>, &'me S, *mut raw::EscapableHandleScope, Box<F>) = chained_callback::<'root, 'me, T, S, F>;
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

fn nest<'root, 'me, T, S, F>(outer: &'me S, f: F) -> T
    where T: Debug,
          S: ScopeInternal<'root>,
          F: FnOnce(&NestedScope<'root>) -> T
{
    let closure: Box<F> = Box::new(f);
    let callback: extern "C" fn(&mut Box<Option<T>>, &'root Realm, Box<F>) = nested_callback::<'root, T, F>;
    let mut result: Box<Option<T>> = Box::new(None);
    {
        let out: &mut Box<Option<T>> = &mut result;
        { *outer.active_cell().borrow_mut() = false; }
        unsafe {
            let out: *mut c_void = mem::transmute(out);
            let closure: *mut c_void = mem::transmute(closure);
            let callback: extern "C" fn(&mut c_void, *mut c_void, *mut c_void) = mem::transmute(callback);
            let realm: *mut c_void = mem::transmute(outer.realm());
            Nan_Nested(out, closure, callback, realm);
        }
        { *outer.active_cell().borrow_mut() = true; }
    }
    result.unwrap()
}

extern "C" fn nested_callback<'root, T, F>(out: &mut Box<Option<T>>,
                                           realm: &'root Realm,
                                           f: Box<F>)
    where T: Debug,
          F: FnOnce(&NestedScope<'root>) -> T
{
    let nested = NestedScope {
        realm: realm,
        active: RefCell::new(true)
    };
    let result = f(&nested);
    **out = Some(result);
}

impl<'root> Scope<'root> for NestedScope<'root> {
    fn realm(&self) -> &'root Realm { self.realm }

    fn active(&self) -> bool {
        *self.active.borrow()
    }

    fn nested<'me, T: Debug, F: FnOnce(&NestedScope<'root>) -> T>(&'me self, f: F) -> T {
        nest(self, f)
    }

    fn chained<'outer, T: Debug, F: FnOnce(&ChainedScope<'root, 'outer>) -> T>(&'outer self, f: F) -> T {
        chain(self, f)
    }
}

impl<'root> ScopeInternal<'root> for NestedScope<'root> {
    fn active_cell(&self) -> &RefCell<bool> {
        &self.active
    }
}

impl<'root, 'parent> Scope<'root> for ChainedScope<'root, 'parent> {
    fn realm(&self) -> &'root Realm { self.realm }

    fn active(&self) -> bool {
        *self.active.borrow()
    }

    fn nested<'me, T: Debug, F: FnOnce(&NestedScope<'root>) -> T>(&'me self, f: F) -> T {
        nest(self, f)
    }

    fn chained<'outer, T: Debug, F: FnOnce(&ChainedScope<'root, 'outer>) -> T>(&'outer self, f: F) -> T {
        chain(self, f)
    }
}

impl<'root, 'parent> ScopeInternal<'root> for ChainedScope<'root, 'parent> {
    fn active_cell(&self) -> &RefCell<bool> {
        &self.active
    }
}
