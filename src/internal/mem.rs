use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use neon_sys;
use neon_sys::raw;
use internal::js::{Value, ValueInternal, SuperType};
use internal::js::error::JsTypeError;
use internal::vm::{JsResult, Lock, LockState};
use internal::scope::Scope;

pub trait Managed: Copy {
    fn to_raw(self) -> raw::Local;

    fn from_raw(h: raw::Local) -> Self;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Handle<'a, T: Managed + 'a> {
    value: T,
    phantom: PhantomData<&'a T>
}

impl<'a, T: Value + 'a> Handle<'a, T> {
    pub fn lock(self) -> LockedHandle<'a, T> {
        LockedHandle::new(self)
    }
}

impl<'a, T: Managed + 'a> PartialEq for Handle<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { neon_sys::mem::same_handle(self.to_raw(), other.to_raw()) }
    }
}

impl<'a, T: Managed + 'a> Eq for Handle<'a, T> { }

pub trait HandleInternal<'a, T: Managed + 'a> {
    fn new(value: T) -> Handle<'a, T>;
}

impl<'a, T: Managed + 'a> HandleInternal<'a, T> for Handle<'a, T> {
    fn new(value: T) -> Handle<'a, T> {
        Handle {
            value: value,
            phantom: PhantomData
        }
    }
}

impl<'a, T: Value> Handle<'a, T> {
    // This method does not require a scope because it only copies a handle.
    pub fn upcast<U: Value + SuperType<T>>(&self) -> Handle<'a, U> {
        Handle::new(SuperType::upcast_internal(self.value))
    }
}

impl<'a, T: Value> Handle<'a, T> {
    pub fn is_a<U: Value>(&self) -> bool {
        U::downcast(self.value).is_some()
    }

    pub fn downcast<U: Value>(&self) -> Option<Handle<'a, U>> {
        U::downcast(self.value).map(Handle::new)
    }

    pub fn check<U: Value>(&self) -> JsResult<'a, U> {
        match U::downcast(self.value) {
            Some(v) => Ok(Handle::new(v)),
            None => JsTypeError::throw("type error")
        }
    }
}

impl<'a, T: Managed> Deref for Handle<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        &self.value
    }
}

impl<'a, T: Managed> DerefMut for Handle<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut self.value
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LockedHandle<'a, T: Value + 'a>(Handle<'a, T>);

unsafe impl<'a, T: Value + 'a> Sync for LockedHandle<'a, T> { }

impl<'a, T: Value + 'a> LockedHandle<'a, T> {
    pub fn new(h: Handle<'a, T>) -> LockedHandle<'a, T> {
        LockedHandle(h)
    }

    pub fn unlock<'b, U: Scope<'b>>(self, _: &mut U) -> Handle<'a, T> { self.0 }
}

impl<'a, T: Value> Lock for LockedHandle<'a, T> {
    type Internals = LockedHandle<'a, T>;

    unsafe fn expose(self, _: &mut LockState) -> Self::Internals {
        self
    }
}
