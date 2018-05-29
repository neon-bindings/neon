//! Types encapsulating _handles_ to managed JavaScript memory.
//!
//! 

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::error::Error;
use std::fmt::{self, Debug, Display};
use neon_runtime;
use neon_runtime::raw;
use js::Value;
use js::internal::SuperType;
use js::error::{JsError, Kind};
use vm::{Context, JsResult, JsResultExt};

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct DowncastError<F: Value, T: Value> {
    phantom_from: PhantomData<F>,
    phantom_to: PhantomData<T>,
    description: String
}

impl<F: Value, T: Value> Debug for DowncastError<F, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "DowncastError")
    }
}

impl<F: Value, T: Value> DowncastError<F, T> {
    fn new() -> Self {
        DowncastError {
            phantom_from: PhantomData,
            phantom_to: PhantomData,
            description: format!("failed downcast to {}", T::name())
        }
    }
}

impl<F: Value, T: Value> Display for DowncastError<F, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl<F: Value, T: Value> Error for DowncastError<F, T> {
    fn description(&self) -> &str {
        &self.description
    }
}

pub type DowncastResult<'a, F, T> = Result<Handle<'a, T>, DowncastError<F, T>>;

impl<'a, F: Value, T: Value> JsResultExt<'a, T> for DowncastResult<'a, F, T> {
    fn unwrap_or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> JsResult<'a, T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => JsError::throw(cx, Kind::TypeError, &e.description)
        }
    }
}

impl<'a, T: Value> Handle<'a, T> {
    // This method does not require a VM context because it only copies a handle.
    pub fn upcast<U: Value + SuperType<T>>(&self) -> Handle<'a, U> {
        Handle::new_internal(SuperType::upcast_internal(self.value))
    }

    pub fn is_a<U: Value>(&self) -> bool {
        U::is_typeof(self.value)
    }

    pub fn downcast<U: Value>(&self) -> DowncastResult<'a, T, U> {
        match U::downcast(self.value) {
            Some(v) => Ok(Handle::new_internal(v)),
            None => Err(DowncastError::new())
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
