use std::{ptr, slice};
use std::marker::PhantomData;
use std::mem;
use std;

#[repr(C)]
#[allow(raw_pointer_derive)]
#[derive(Copy, Clone)]
pub struct Buf<'a> {
    ptr: *mut u8,
    len: usize,
    marker: PhantomData<&'a ()>
}

impl<'a> Buf<'a> {
    pub unsafe fn uninitialized<'b>() -> Buf<'b> {
        Buf {
            ptr: mem::uninitialized(),
            len: mem::uninitialized(),
            marker: PhantomData
        }
    }

    pub fn wrap(s: &'a str) -> Buf<'a> {
        Buf {
            ptr: s.as_ptr() as *mut u8,
            len: s.len(),
            marker: PhantomData,
        }
    }

    // FIXME: this is not legit; can't use unchecked without making this method unsafe
    pub fn as_str(self) -> Option<&'a str> {
        if self.ptr == ptr::null_mut() {
            return None;
        }

        unsafe {
            let s = slice::from_raw_parts(self.ptr as *const u8, self.len);
            Some(std::str::from_utf8_unchecked(s))
        }
    }

    pub fn as_slice(&self) -> Option<&'a [u8]> {
        if self.ptr.is_null() {
            return None;
        }

        unsafe {
            Some(slice::from_raw_parts(self.ptr, self.len))
        }
    }

    pub fn as_mut_slice(&mut self) -> Option<&'a mut [u8]> {
        if self.ptr.is_null() {
            return None;
        }

        unsafe {
            Some(slice::from_raw_parts_mut(self.ptr, self.len))
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }
}
