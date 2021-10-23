use crate::borrow::LoanError;
use std;
use std::collections::HashSet;
use std::os::raw::c_void;

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
    mutable_loans: HashSet<*const c_void>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            immutable_loans: HashSet::new(),
            mutable_loans: HashSet::new(),
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
