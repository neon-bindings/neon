//! Types encapsulating _handles_ to managed JavaScript memory.
//!
//! 

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use neon_runtime;
use neon_runtime::raw;
use js::Value;
use js::internal::SuperType;
use js::error::{JsError, Kind};
use vm::JsResult;

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

impl<'a, T: Managed + 'a> PartialEq for Handle<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { neon_runtime::mem::same_handle(self.to_raw(), other.to_raw()) }
    }
}

impl<'a, T: Managed + 'a> Eq for Handle<'a, T> { }

impl<'a, T: Managed + 'a> Handle<'a, T> {
    pub(crate) fn new_internal(value: T) -> Handle<'a, T> {
        Handle {
            value: value,
            phantom: PhantomData
        }
    }
}

impl<'a, T: Value> Handle<'a, T> {
    // This method does not require a scope because it only copies a handle.
    pub fn upcast<U: Value + SuperType<T>>(&self) -> Handle<'a, U> {
        Handle::new_internal(SuperType::upcast_internal(self.value))
    }

    pub fn is_a<U: Value>(&self) -> bool {
        U::downcast(self.value).is_some()
    }

    pub fn downcast<U: Value>(&self) -> Option<Handle<'a, U>> {
        U::downcast(self.value).map(Handle::new_internal)
    }

    pub fn check<U: Value>(&self) -> JsResult<'a, U> {
        match U::downcast(self.value) {
            Some(v) => Ok(Handle::new_internal(v)),
            None => JsError::throw(Kind::TypeError, "type error")
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
