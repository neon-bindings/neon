use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use neon_sys;
use internal::value::{Any, AnyInternal, SuperType};
use internal::error::TypeError;
use internal::vm::{JS, Lock, LockState};
use internal::scope::Scope;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Handle<'a, T: Any + 'a> {
    value: T,
    phantom: PhantomData<&'a T>
}

impl<'a, T: Any + 'a> Handle<'a, T> {
    pub fn lock(self) -> LockedHandle<'a, T> {
        LockedHandle::new(self)
    }
}

impl<'a, T: Any + 'a> PartialEq for Handle<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { neon_sys::mem::SameHandle(self.to_raw(), other.to_raw()) }
    }
}

impl<'a, T: Any + 'a> Eq for Handle<'a, T> { }

pub trait HandleInternal<'a, T: Any + 'a> {
    fn new(value: T) -> Handle<'a, T>;
}

impl<'a, T: Any + 'a> HandleInternal<'a, T> for Handle<'a, T> {
    fn new(value: T) -> Handle<'a, T> {
        Handle {
            value: value,
            phantom: PhantomData
        }
    }
}

impl<'a, T: Any> Handle<'a, T> {
    // This method does not require a scope because it only copies a handle.
    pub fn upcast<U: Any + SuperType<T>>(&self) -> Handle<'a, U> {
        Handle::new(SuperType::upcast_internal(self.value))
    }
}

impl<'a, T: Any> Handle<'a, T> {
    pub fn downcast<U: Any>(&self) -> Option<Handle<'a, U>> {
        U::downcast(self.value).map(Handle::new)
    }

    pub fn check<U: Any>(&self) -> JS<'a, U> {
        match U::downcast(self.value) {
            Some(v) => Ok(Handle::new(v)),
            None => TypeError::throw("type error")
        }
    }
}

impl<'a, T: Any> Deref for Handle<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        &self.value
    }
}

impl<'a, T: Any> DerefMut for Handle<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut self.value
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LockedHandle<'a, T: Any + 'a>(Handle<'a, T>);

unsafe impl<'a, T: Any + 'a> Sync for LockedHandle<'a, T> { }

impl<'a, T: Any + 'a> LockedHandle<'a, T> {
    pub fn new(h: Handle<'a, T>) -> LockedHandle<'a, T> {
        LockedHandle(h)
    }

    pub fn unlock<'b, U: Scope<'b>>(self, _: &mut U) -> Handle<'a, T> { self.0 }
}

impl<'a, T: Any> Lock for LockedHandle<'a, T> {
    type Internals = LockedHandle<'a, T>;

    unsafe fn expose(self, _: &mut LockState) -> Self::Internals {
        self
    }
}
