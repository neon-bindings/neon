use std::ops::{Index, IndexMut};
use std::str;
use std::str::Utf8Error;

use vm::Throw;
use internal::error::TypeError;
use internal::value::{SomeObject, Any, AnyInternal, Object, build};
use internal::mem::Handle;
use nanny_sys::raw;
use nanny_sys::{Nan_NewBuffer, Node_Buffer_Data, Node_Buffer_Value_HasInstance};
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
    pub fn new<'a, T: Scope<'a>>(_: &mut T, size: u32) -> Result<Handle<'a, SomeObject>, Throw> {
        build(|out| { unsafe { Nan_NewBuffer(out, size) } })
    }

    pub fn data(&self) -> Buf {
        unsafe {
            let mut result = Buf::uninitialized();
            Node_Buffer_Data(&mut result, self.to_raw());
            result
        }
    }

    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(self.data().as_slice().unwrap())
    }

    pub fn check_str(&self) -> Result<&str, Throw> {
        self.as_str().map_err(|_| {
            TypeError::throw::<()>("buffer contents are invalid UTF-8").err().unwrap()
        })
    }
}

impl AnyInternal for Buffer {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { Buffer(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { Node_Buffer_Value_HasInstance(other.to_raw()) }
    }
}

impl Any for Buffer { }

impl Object for Buffer { }
