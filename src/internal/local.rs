use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use nanny_sys::raw;
use internal::value::{Value, ValueInternal, Any, AnyInternal};

#[repr(C)]
pub struct Local<'a, T: Clone + Value + 'a> {
    value: T,
    phantom: PhantomData<&'a T>
}

pub trait LocalInternal<'a, T: Clone + Value + 'a> {
    fn new(value: T) -> Local<'a, T>;
    fn to_raw_mut_ref(&mut self) -> &mut raw::Local;
}

impl<'a, T: Clone + Value + 'a> LocalInternal<'a, T> for Local<'a, T> {
    fn new(value: T) -> Local<'a, T> {
        Local {
            value: value,
            phantom: PhantomData
        }
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        match self {
            &mut Local { ref mut value, .. } => {
                value.to_raw_mut_ref()
            }
        }
    }
}

impl<'a, T: Clone + Value> Local<'a, T> {
    pub fn upcast(&self) -> Local<'a, Any> {
        Any::new(self.value.to_raw())
    }
}

impl<'a, T: Clone + Value> Deref for Local<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        &self.value
    }
}

impl<'a, T: Clone + Value> DerefMut for Local<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut self.value
    }
}
