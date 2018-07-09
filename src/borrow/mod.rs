//! Types and traits for obtaining temporary access to the internals of JavaScript values.

use std::ops::{Deref, DerefMut, Drop};
use std::fmt;
use std::os::raw::c_void;

use cx::Lock;
use self::internal::Pointer;

pub(crate) mod internal {
    use std;
    use std::os::raw::c_void;
    use std::collections::HashSet;
    use borrow::LoanError;

    pub unsafe trait Pointer {
        unsafe fn as_ptr(&self) -> *const c_void;
        unsafe fn as_mut(&mut self) -> *mut c_void;
    }

    unsafe impl<T> Pointer for *mut T {
        unsafe fn as_ptr(&self) -> *const c_void {
            *self as *const c_void
        }

        unsafe fn as_mut(&mut self) -> *mut c_void {
            *self as *mut c_void
        }
    }
    unsafe impl<'a, T> Pointer for &'a mut T {
        unsafe fn as_ptr(&self) -> *const c_void {
            let r: &T = &**self;
            std::mem::transmute(r)
        }

        unsafe fn as_mut(&mut self) -> *mut c_void {
            let r: &mut T = &mut **self;
            std::mem::transmute(r)
        }
    }

    pub struct Ledger {
        immutable_loans: HashSet<*const c_void>,
        mutable_loans: HashSet<*const c_void>
    }

    impl Ledger {
        pub fn new() -> Self {
            Ledger {
                immutable_loans: HashSet::new(),
                mutable_loans: HashSet::new()
            }
        }

        pub fn try_borrow<T>(&mut self, p: *const T) -> Result<(), LoanError> {
            let p = p as *const c_void;
            if self.mutable_loans.contains(&p) {
                return Err(LoanError::Mutating(p));
            }
            self.immutable_loans.insert(p);
            Ok(())
        }

        pub fn settle<T>(&mut self, p: *const T) {
            let p = p as *const c_void;
            self.immutable_loans.remove(&p);
        }

        pub fn try_borrow_mut<T>(&mut self, p: *mut T) -> Result<(), LoanError> {
            let p = p as *const c_void;
            if self.mutable_loans.contains(&p) {
                return Err(LoanError::Mutating(p));
            } else if self.immutable_loans.contains(&p) {
                return Err(LoanError::Frozen(p));
            }
            self.mutable_loans.insert(p);
            Ok(())
        }

        pub fn settle_mut<T>(&mut self, p: *mut T) {
            let p = p as *const c_void;
            self.mutable_loans.remove(&p);
        }
    }
}

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
            Err(e) => panic!("{}", e)
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
            Err(e) => panic!("{}", e)
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
    Frozen(*const c_void)

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
    lock: &'a Lock<'a>
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
    lock: &'a Lock<'a>
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
