use std::{
    cell::RefCell,
    error::Error,
    fmt::{self, Debug, Display},
    ops::{Deref, DerefMut},
};

use crate::{
    context::Context,
    result::{NeonResult, ResultExt},
    types::buffer::lock::{Ledger, Lock},
};

pub(crate) mod lock;
pub(super) mod types;

/// A trait for borrowing binary data from JavaScript values
///
/// Provides both statically and dynamically checked borrowing. Mutable borrows
/// are guaranteed not to overlap with other borrows.
pub trait TypedArray: private::Sealed {
    type Item;

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
        &self.data
    }
}

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        let mut ledger = self.ledger.borrow_mut();
        let range = Ledger::slice_to_range(&self.data);
        let i = ledger.shared.iter().rposition(|r| r == &range).unwrap();

        ledger.shared.remove(i);
    }
}

impl<'a, T> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        let mut ledger = self.ledger.borrow_mut();
        let range = Ledger::slice_to_range(&self.data);
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

mod private {
    pub trait Sealed {}
}
