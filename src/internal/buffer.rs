use std::ops::{Index, IndexMut};
use std::str;
use std::str::Utf8Error;

use vm::Throw;
use internal::value::{Value, Object, ObjectInternal, Tagged, TaggedInternal};
use internal::mem::Handle;
use nanny_sys::raw;
use nanny_sys::{Nan_NewBuffer, Node_Buffer_Data, Node_Buffer_Value_HasInstance, Node_Buffer_Object_HasInstance};
use scope::Scope;
use nanny_sys::buf::Buf;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Buffer(raw::Local);

impl Index<usize> for Buffer {
    type Output = u8;
    fn index<'a>(&'a self, index: usize) -> &'a u8 {
        self.data().as_slice().unwrap().index(index)
    }
}

impl IndexMut<usize> for Buffer {
    fn index_mut<'a>(&'a mut self, index: usize) -> &mut u8 {
        self.data().as_mut_slice().unwrap().index_mut(index)
    }
}

impl Buffer {
    pub fn new<'fun, 'block, T: Scope<'fun, 'block>>(_: &mut T, size: u32) -> Option<Handle<'block, Object>> {
        Object::build_opt(|out| { unsafe { Nan_NewBuffer(out, size) } })
    }

    pub fn data(&self) -> Buf {
        unsafe {
            let mut result = Buf::uninitialized();
            Node_Buffer_Data(&mut result, self.to_raw_ref());
            result
        }
    }

    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(self.data().as_slice().unwrap())
    }

    pub fn check_str(&self) -> Result<&str, Throw> {
        self.as_str().map_err(|_| {
            // FIXME: throw a type error
            Throw
        })
    }
}

impl Value {
    pub fn as_buffer<'fun, 'block, 'scope, T: Scope<'fun, 'block>>(&self, _: &'scope mut T) -> Option<Handle<'block, Buffer>> {
        if unsafe { Node_Buffer_Value_HasInstance(self.to_raw_ref()) } {
            Some(self.cast(Buffer))
        } else {
            None
        }
    }
}

impl Object {
    pub fn as_buffer<'fun, 'block, 'scope, T: Scope<'fun, 'block>>(&self, _: &'scope mut T) -> Option<Handle<'block, Buffer>> {
        if unsafe { Node_Buffer_Object_HasInstance(self.to_raw_ref()) } {
            Some(self.cast(Buffer))
        } else {
            None
        }
    }

    pub fn check_buffer<'fun, 'block, 'scope, T: Scope<'fun, 'block>>(&self, scope: &'scope mut T) -> Result<Handle<'block, Buffer>, Throw> {
        self.as_buffer(scope).ok_or_else(|| {
            // FIXME: throw a type error
            Throw
        })
    }
}

impl TaggedInternal for Buffer {
    fn to_raw_mut_ref(&mut self) -> &mut raw::Local {
        let &mut Buffer(ref mut local) = self;
        local
    }

    fn to_raw_ref(&self) -> &raw::Local {
        let &Buffer(ref local) = self;
        local
    }
}

impl Tagged for Buffer { }
