use std::{ptr, slice};
use std::marker::PhantomData;
use std::mem;
use std::str;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Buf<'a> {
    ptr: *mut u8,
    len: usize,
    marker: PhantomData<&'a ()>
}

impl<'a> Buf<'a> {
    pub fn wrap(s: &'a str) -> Buf<'a> {
        Buf {
            ptr: s.as_ptr() as *mut u8,
            len: s.len(),
            marker: PhantomData,
        }
    }

    pub fn as_str(self) -> Option<&'a str> {
        if self.ptr == ptr::null_mut() {
            return None;
        }

        unsafe {
            let s = slice::from_raw_parts(self.ptr as *const u8, self.len);
            str::from_utf8(s).ok()
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

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }
}

unsafe impl<'a> Sync for Buf<'a> { }
