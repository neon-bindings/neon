use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use nanny_sys::raw;
use internal::value::{Tagged, TaggedInternal, Value, ValueInternal};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Handle<'a, T: Tagged + 'a> {
    value: T,
    phantom: PhantomData<&'a T>
}

pub trait HandleInternal<'a, T: Tagged + 'a> {
    fn new(value: T) -> Handle<'a, T>;
    fn to_raw_mut_ref(&mut self) -> &mut raw::Local;
}

impl<'a, T: Tagged + 'a> HandleInternal<'a, T> for Handle<'a, T> {
    fn new(value: T) -> Handle<'a, T> {
        Handle {
            value: value,
            phantom: PhantomData
        }
    }

    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        match self {
            &mut Handle { ref mut value, .. } => {
                value.to_raw_mut_ref()
            }
        }
    }
}

impl<'a, T: Tagged> Handle<'a, T> {
    pub fn upcast(&self) -> Handle<'a, Value> {
        Value::new_internal(self.value.to_raw())
    }
}

impl<'a, T: Tagged> Deref for Handle<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        &self.value
    }
}

impl<'a, T: Tagged> DerefMut for Handle<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut self.value
    }
}
