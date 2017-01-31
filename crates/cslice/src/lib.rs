//! A library of _C-slices_: slices with a stable ABI for interfacing with C.
//!
//! This library provides two types, `CSlice` and `CMutSlice`, for communicating
//! with C about Rust slices or foreign slice-like data structures. Both types
//! have a stable ABI consisting of exactly two pointer-sized words:
//!
//! ```c
//! struct {
//!     void *base;
//!     size_t len;
//! }
//! ```
//!
//! C-slices and Rust slices are interchangeable, with conversion methods in both
//! directions.
//!
//! This makes it possible to construct slices from foreign code, as well as to
//! communicate Rust slices to foreign code conveniently.
#![no_std]

use core::{ptr, slice};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

/// An immutable slice, equivalent to `&'a T`.
///
/// A `CSlice` can be constructed from a corresponding Rust slice via the `AsCSlice` trait.
///
/// A Rust slice can be constructed from a corresponding `CSlice` via `as_ref`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CSlice<'a, T> {
    base: *const T,
    len: usize,
    marker: PhantomData<&'a ()>
}

impl<'a, T> CSlice<'a, T> {
    /// Create a `CSlice` from raw data.
    ///
    /// # Safety
    ///
    /// The region of memory from `base` (inclusive) to `base + len * sizeof<T>`
    /// (exclusive) must be valid for the duration of lifetime `'a`.
    pub unsafe fn new(base: *const T, len: usize) -> Self {
        assert!(base != ptr::null());
        CSlice {
            base: base,
            len: len,
            marker: PhantomData
        }
    }

    /// Produces a raw pointer to the slice's buffer.
    pub fn as_ptr(&self) -> *const T {
        self.base
    }

    /// Returns the number of elements in the slice.
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> AsRef<[T]> for CSlice<'a, T> {
    fn as_ref(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.base, self.len)
        }
    }
}

/// A mutable slice, equivalent to `&'a mut T`.
///
/// A `CMutSlice` can be constructed from a corresponding Rust slice via the `AsCMutSlice` trait.
///
/// A Rust slice can be constructed from a corresponding `CMutSlice` via `as_mut`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CMutSlice<'a, T> {
    base: *mut T,
    len: usize,
    marker: PhantomData<&'a ()>
}

impl<'a, T> CMutSlice<'a, T> {
    /// Create a `CSlice` from raw data.
    ///
    /// # Safety
    ///
    /// The region of memory from `base` (inclusive) to `base + len * sizeof<T>`
    /// (exclusive) must be valid for the duration of lifetime `'a`.
    pub unsafe fn new(base: *mut T, len: usize) -> Self {
        assert!(base != ptr::null_mut());
        CMutSlice {
            base: base,
            len: len,
            marker: PhantomData
        }
    }

    /// Produces a raw pointer to the slice's buffer.
    pub fn as_ptr(&self) -> *const T {
        self.base
    }

    /// Produces a raw pointer to the slice's buffer.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.base
    }

    /// A cheap conversion to a Rust slice. This is slightly more general than `as_ref`.
    pub fn as_slice(&self) -> &'a [T] {
        unsafe {
            slice::from_raw_parts(self.base, self.len)
        }
    }

    /// A cheap conversion to a mutable Rust slice. This is slightly more general than `as_mut`.
    pub fn as_mut_slice(&mut self) -> &'a mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.base, self.len)
        }
    }

    /// Returns the number of elements in the slice.
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> AsRef<[T]> for CMutSlice<'a, T> {
    fn as_ref(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.base, self.len)
        }
    }
}

impl<'a, T> AsMut<[T]> for CMutSlice<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.base, self.len)
        }
    }
}

unsafe impl<'a, T> Sync for CSlice<'a, T> { }

unsafe impl<'a, T> Sync for CMutSlice<'a, T> { }


impl<'a, T> Index<usize> for CSlice<'a, T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        self.as_ref().index(i)
    }
}

impl<'a, T> Index<usize> for CMutSlice<'a, T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        self.as_ref().index(i)
    }
}

impl<'a, T> IndexMut<usize> for CMutSlice<'a, T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        self.as_mut().index_mut(i)
    }
}

/// A cheap conversion to a `CSlice`.
pub trait AsCSlice<'a, T> {
    /// Performs the conversion.
    fn as_c_slice(&'a self) -> CSlice<'a, T>;
}

/// A cheap conversion to a `CMutSlice`.
pub trait AsCMutSlice<'a, T> {
    /// Performs the conversion.
    fn as_c_mut_slice(&'a mut self) -> CMutSlice<'a, T>;
}

impl<'a> AsCSlice<'a, u8> for str {
    fn as_c_slice(&'a self) -> CSlice<'a, u8> {
        CSlice {
            base: self.as_ptr(),
            len: self.len(),
            marker: PhantomData
        }
    }
}

impl<'a, T> AsCSlice<'a, T> for [T] {
    fn as_c_slice(&'a self) -> CSlice<'a, T> {
        CSlice {
            base: self.as_ptr(),
            len: self.len(),
            marker: PhantomData
        }
    }
}

impl<'a, T> AsCMutSlice<'a, T> for [T] {
    fn as_c_mut_slice(&'a mut self) -> CMutSlice<'a, T> {
        CMutSlice {
            base: self.as_mut_ptr(),
            len: self.len(),
            marker: PhantomData
        }
    }
}
