//! Types and traits for working with binary buffers.

use std::{
    cell::RefCell,
    error::Error,
    fmt::{self, Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    context::Context,
    handle::Handle,
    result::{JsResult, NeonResult, ResultExt},
    types::{
        buffer::lock::{Ledger, Lock},
        JsArrayBuffer, JsTypedArray, Value,
    },
};

pub(crate) mod lock;
pub(super) mod types;

pub use types::Binary;

/// A trait allowing Rust to borrow binary data from the memory buffer of JavaScript
/// [typed arrays][typed-arrays].
///
/// This trait provides both statically and dynamically checked borrowing. As usual
/// in Rust, mutable borrows are guaranteed not to overlap with other borrows.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// use neon::types::buffer::TypedArray;
///
/// fn double(mut cx: FunctionContext) -> JsResult<JsUndefined> {
///     let mut array: Handle<JsUint32Array> = cx.argument(0)?;
///
///     for elem in array.as_mut_slice(&mut cx).iter_mut() {
///         *elem *= 2;
///     }
///
///     Ok(cx.undefined())
/// }
/// ```
///
/// [typed-arrays]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Typed_arrays
pub trait TypedArray: Value {
    type Item: Binary;

    /// Statically checked immutable borrow of binary data.
    ///
    /// This may not be used if a mutable borrow is in scope. For the dynamically
    /// checked variant see [`TypedArray::try_borrow`].
    fn as_slice<'cx, 'a, C>(&self, cx: &'a C) -> &'a [Self::Item]
    where
        C: Context<'cx>;

    /// Statically checked mutable borrow of binary data.
    ///
    /// This may not be used if any other borrow is in scope. For the dynamically
    /// checked variant see [`TypedArray::try_borrow_mut`].
    fn as_mut_slice<'cx, 'a, C>(&mut self, cx: &'a mut C) -> &'a mut [Self::Item]
    where
        C: Context<'cx>;

    /// Dynamically checked immutable borrow of binary data, returning an error if the
    /// the borrow would overlap with a mutable borrow.
    ///
    /// The borrow lasts until [`Ref`] exits scope.
    ///
    /// This is the dynamically checked version of [`TypedArray::as_slice`].
    fn try_borrow<'cx, 'a, C>(&self, lock: &'a Lock<C>) -> Result<Ref<'a, Self::Item>, BorrowError>
    where
        C: Context<'cx>;

    /// Dynamically checked mutable borrow of binary data, returning an error if the
    /// the borrow would overlap with an active borrow.
    ///
    /// The borrow lasts until [`RefMut`] exits scope.
    ///
    /// This is the dynamically checked version of [`TypedArray::as_mut_slice`].
    fn try_borrow_mut<'cx, 'a, C>(
        &mut self,
        lock: &'a Lock<C>,
    ) -> Result<RefMut<'a, Self::Item>, BorrowError>
    where
        C: Context<'cx>;

    /// Returns the size, in bytes, of the allocated binary data.
    fn size<'cx, C>(&self, cx: &mut C) -> usize
    where
        C: Context<'cx>;

    /// Constructs an instance from a slice by copying its contents.
    fn from_slice<'cx, C>(cx: &mut C, slice: &[Self::Item]) -> JsResult<'cx, Self>
    where
        C: Context<'cx>;
}

#[derive(Debug)]
/// Wraps binary data immutably borrowed from a JavaScript value.
pub struct Ref<'a, T> {
    data: &'a [T],
    ledger: &'a RefCell<Ledger>,
}

#[derive(Debug)]
/// Wraps binary data mutably borrowed from a JavaScript value.
pub struct RefMut<'a, T> {
    data: &'a mut [T],
    ledger: &'a RefCell<Ledger>,
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        let mut ledger = self.ledger.borrow_mut();
        let range = Ledger::slice_to_range(self.data);
        let i = ledger.shared.iter().rposition(|r| r == &range).unwrap();

        ledger.shared.remove(i);
    }
}

impl<'a, T> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        let mut ledger = self.ledger.borrow_mut();
        let range = Ledger::slice_to_range(self.data);
        let i = ledger.owned.iter().rposition(|r| r == &range).unwrap();

        ledger.owned.remove(i);
    }
}

#[derive(Eq, PartialEq)]
/// An error returned by [`TypedArray::try_borrow`] or [`TypedArray::try_borrow_mut`] indicating
/// that a mutable borrow would overlap with another borrow.
///
/// [`BorrowError`] may be converted to an exception with [`ResultExt::or_throw`].
pub struct BorrowError {
    _private: (),
}

impl BorrowError {
    fn new() -> Self {
        BorrowError { _private: () }
    }
}

impl Error for BorrowError {}

impl Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Borrow overlaps with an active mutable borrow", f)
    }
}

impl Debug for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowError").finish()
    }
}

impl<T> ResultExt<T> for Result<T, BorrowError> {
    fn or_throw<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<T> {
        self.or_else(|_| cx.throw_error("BorrowError"))
    }
}

/// Represents a typed region of an [`ArrayBuffer`](crate::types::JsArrayBuffer).
///
/// A `Region` can be created via the
/// [`Handle<JsArrayBuffer>::region()`](crate::handle::Handle::region) or
/// [`JsTypedArray::region()`](crate::types::JsTypedArray::region) methods.
///
/// A region is **not** checked for validity until it is converted to
/// a typed array via [`to_typed_array()`](Region::to_typed_array) or
/// [`JsTypedArray::from_region()`](crate::types::JsTypedArray::from_region).
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// # fn f(mut cx: FunctionContext) -> JsResult<JsUint32Array> {
/// // Allocate a 16-byte ArrayBuffer and a uint32 array of length 2 (i.e., 8 bytes)
/// // starting at byte offset 4 of the buffer:
/// //
/// //       0       4       8       12      16
/// //      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// // buf: | | | | | | | | | | | | | | | | |
/// //      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// //               ^       ^
/// //               |       |
/// //              +-------+-------+
/// //         arr: |       |       |
/// //              +-------+-------+
/// //               0       1       2
/// let buf = cx.array_buffer(16)?;
/// let arr = JsUint32Array::from_region(&mut cx, &buf.region(4, 2))?;
/// # Ok(arr)
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct Region<'cx, T: Binary> {
    buffer: Handle<'cx, JsArrayBuffer>,
    offset: usize,
    len: usize,
    phantom: PhantomData<T>,
}

impl<'cx, T: Binary> Region<'cx, T> {
    /// Returns the handle to the region's buffer.
    pub fn buffer(&self) -> Handle<'cx, JsArrayBuffer> {
        self.buffer
    }

    /// Returns the starting byte offset of the region.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the number of elements of type `T` in the region.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the size of the region in bytes, which is equal to
    /// `(self.len() * size_of::<T>())`.
    pub fn size(&self) -> usize {
        self.len * std::mem::size_of::<T>()
    }

    /// Constructs a typed array for this buffer region.
    ///
    /// The resulting typed array has `self.len()` elements and a size of
    /// `self.size()` bytes.
    ///
    /// Throws an exception if the region is invalid, for example if the starting
    /// offset is not properly aligned, or the length goes beyond the end of the
    /// buffer.
    pub fn to_typed_array<'c, C>(&self, cx: &mut C) -> JsResult<'c, JsTypedArray<T>>
    where
        C: Context<'c>,
    {
        JsTypedArray::from_region(cx, self)
    }
}

mod private {
    use super::Binary;
    use crate::sys::raw;
    use std::fmt::{Debug, Formatter};
    use std::marker::PhantomData;

    pub trait Sealed {}

    #[derive(Clone)]
    pub struct JsTypedArrayInner<T: Binary> {
        pub(super) local: raw::Local,
        pub(super) buffer: raw::Local,
        pub(super) _type: PhantomData<T>,
    }

    impl<T: Binary> Debug for JsTypedArrayInner<T> {
        fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
            f.write_str("JsTypedArrayInner { ")?;
            f.write_str("local: ")?;
            self.local.fmt(f)?;
            f.write_str(", buffer: ")?;
            self.buffer.fmt(f)?;
            f.write_str(", _type: PhantomData")?;
            f.write_str(" }")?;
            Ok(())
        }
    }

    impl<T: Binary> Copy for JsTypedArrayInner<T> {}
}
