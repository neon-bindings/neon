//! Provides temporary access to JavaScript typed arrays.
//!
//! ## Typed Arrays
//!
//! JavaScript's [typed arrays][typed-arrays] are objects that allow reading and writing
//! raw binary data in memory.
//!
//! Typed arrays are managed with the [`ArrayBuffer`][ArrayBuffer] type, which controls
//! the storage of the underlying data buffer, and several typed views for managing access
//! to the buffer. Neon provides access to the `ArrayBuffer` class with the
//! [`JsArrayBuffer`](crate::types::JsArrayBuffer) type.
//!
//! Node also provides a [`Buffer`][Buffer] type, which is built on top of `ArrayBuffer`
//! and provides additional functionality. Neon provides access to the `Buffer` class
//! with the [`JsBuffer`](crate::types::JsBuffer) type.
//!
//! Many of Node's I/O APIs work with these types, and they can also be used for
//! compact in-memory data structures, which can be shared efficiently between
//! JavaScript and Rust without copying.
//!
//! ## Borrowing
//!
//! Neon makes it possible to [borrow][borrow] temporary access to the internal memory
//! of a typed array by pausing execution of JavaScript with a
//! [`Lock`](crate::context::Lock) and returning a reference to a
//! [`BinaryData`](crate::types::BinaryData) struct. The [`Borrow`](Borrow) and
//! [`BorrowMut`](BorrowMut) traits provide the methods for borrowing this typed array data.
//!
//! [typed-arrays]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Typed_arrays
//! [borrow]: https://doc.rust-lang.org/beta/rust-by-example/scope/borrow.html
//! [ArrayBuffer]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer
//! [Buffer]: https://nodejs.org/api/buffer.html

pub(crate) mod internal;

use std::fmt;
use std::ops::{Deref, DerefMut, Drop};
use std::os::raw::c_void;

use self::internal::Pointer;
use context::Lock;

/// A trait for JS values whose internal contents can be borrowed immutably by Rust while the JS engine is locked.
pub trait Borrow: Sized {
    /// The type of the value's internal contents.
    type Target: Pointer;

    /// Borrow the contents of this value immutably.
    ///
    /// If there is already an outstanding mutable loan for this value, this method panics.
    fn borrow<'a>(self, lock: &'a Lock<'a>) -> Ref<'a, Self::Target> {
        match self.try_borrow(lock) {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        }
    }

    /// Borrow the contents of this value immutably.
    ///
    /// If there is already an outstanding mutable loan for this value, this method fails with a `LoanError`.
    fn try_borrow<'a>(self, lock: &'a Lock<'a>) -> Result<Ref<'a, Self::Target>, LoanError>;
}

/// A trait for JS values whose internal contents can be borrowed mutably by Rust while the JS engine is locked.
pub trait BorrowMut: Borrow {
    /// Borrow the contents of this value mutably.
    ///
    /// If there is already an outstanding loan for this value, this method panics.
    fn borrow_mut<'a>(self, lock: &'a Lock<'a>) -> RefMut<'a, Self::Target> {
        match self.try_borrow_mut(lock) {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        }
    }

    /// Borrow the contents of this value mutably.
    ///
    /// If there is already an outstanding loan for this value, this method panics.
    fn try_borrow_mut<'a>(self, lock: &'a Lock<'a>) -> Result<RefMut<'a, Self::Target>, LoanError>;
}

/// An error produced by a failed loan in the `Borrow` or `BorrowMut` traits.
pub enum LoanError {
    /// Indicates that there is already an outstanding mutable loan for the object at this address.
    Mutating(*const c_void),

    /// Indicates that there is already an outstanding immutable loan for the object at this address.
    Frozen(*const c_void),
}

impl fmt::Display for LoanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LoanError::Mutating(p) => {
                write!(f, "outstanding mutable loan exists for object at {:?}", p)
            }
            LoanError::Frozen(p) => {
                write!(f, "object at {:?} is frozen", p)
            }
        }
    }
}

/// An immutable reference to the contents of a borrowed JS value.
pub struct Ref<'a, T: Pointer> {
    pointer: T,
    lock: &'a Lock<'a>,
}

impl<'a, T: Pointer> Ref<'a, T> {
    pub(crate) unsafe fn new(lock: &'a Lock<'a>, pointer: T) -> Result<Self, LoanError> {
        let mut ledger = lock.ledger.borrow_mut();
        ledger.try_borrow(pointer.as_ptr())?;
        Ok(Ref { pointer, lock })
    }
}

impl<'a, T: Pointer> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        let mut ledger = self.lock.ledger.borrow_mut();
        ledger.settle(unsafe { self.pointer.as_ptr() });
    }
}

impl<'a, T: Pointer> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.pointer
    }
}

/// A mutable reference to the contents of a borrowed JS value.
pub struct RefMut<'a, T: Pointer> {
    pointer: T,
    lock: &'a Lock<'a>,
}

impl<'a, T: Pointer> RefMut<'a, T> {
    pub(crate) unsafe fn new(lock: &'a Lock<'a>, mut pointer: T) -> Result<Self, LoanError> {
        let mut ledger = lock.ledger.borrow_mut();
        ledger.try_borrow_mut(pointer.as_mut())?;
        Ok(RefMut { pointer, lock })
    }
}

impl<'a, T: Pointer> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        let mut ledger = self.lock.ledger.borrow_mut();
        ledger.settle_mut(unsafe { self.pointer.as_mut() });
    }
}

impl<'a, T: Pointer> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.pointer
    }
}

impl<'a, T: Pointer> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pointer
    }
}
