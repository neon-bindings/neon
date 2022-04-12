use std::{cell::RefCell, ops::Range};

use crate::{
    context::Context,
    types::buffer::{BorrowError, Ref, RefMut},
};

#[derive(Debug)]
/// A temporary lock of an execution context.
///
/// While a lock is alive, no JavaScript code can be executed in the execution context.
///
/// Values that support the `Borrow` trait may be dynamically borrowed by passing a
/// [`Lock`].
pub struct Lock<'cx, C> {
    pub(super) cx: &'cx C,
    pub(super) ledger: RefCell<Ledger>,
}

impl<'a: 'cx, 'cx, C> Lock<'cx, C>
where
    C: Context<'a>,
{
    /// Constructs a new [`Lock`] and locks the VM. See also [`Context::lock`].
    pub fn new(cx: &'cx mut C) -> Lock<'cx, C> {
        Lock {
            cx,
            ledger: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
// Bookkeeping for dynamically check borrowing rules
//
// Ranges are open on the end: `[start, end)`
pub(super) struct Ledger {
    // Mutable borrows. Should never overlap with other borrows.
    pub(super) owned: Vec<Range<*const u8>>,

    // Immutable borrows. May overlap or contain duplicates.
    pub(super) shared: Vec<Range<*const u8>>,
}

impl Ledger {
    // Convert a slice of arbitrary type and size to a range of bytes addresses
    //
    // Alignment does not matter because we are only interested in bytes.
    pub(super) fn slice_to_range<T>(data: &[T]) -> Range<*const u8> {
        let Range { start, end } = data.as_ptr_range();

        (start.cast())..(end.cast())
    }

    // Dynamically check a slice conforms to borrow rules before returning by
    // using interior mutability of the ledger.
    pub(super) fn try_borrow<'a, T>(
        ledger: &'a RefCell<Self>,
        data: &'a [T],
    ) -> Result<Ref<'a, T>, BorrowError> {
        ledger.borrow_mut().try_add_borrow(data)?;

        Ok(Ref { ledger, data })
    }

    // Dynamically check a mutable slice conforms to borrow rules before returning by
    // using interior mutability of the ledger.
    pub(super) fn try_borrow_mut<'a, T>(
        ledger: &'a RefCell<Self>,
        data: &'a mut [T],
    ) -> Result<RefMut<'a, T>, BorrowError> {
        ledger.borrow_mut().try_add_borrow_mut(data)?;

        Ok(RefMut { ledger, data })
    }

    // Try to add an immutable borrow to the ledger
    fn try_add_borrow<T>(&mut self, data: &[T]) -> Result<(), BorrowError> {
        let range = Self::slice_to_range(data);

        // Check if the borrow overlaps with any active mutable borrow
        check_overlap(&self.owned, &range)?;

        // Record a record of the immutable borrow
        self.shared.push(range);

        Ok(())
    }

    // Try to add a mutable borrow to the ledger
    fn try_add_borrow_mut<T>(&mut self, data: &mut [T]) -> Result<(), BorrowError> {
        let range = Self::slice_to_range(data);

        // Check if the borrow overlaps with any active mutable borrow
        check_overlap(&self.owned, &range)?;

        // Check if the borrow overlaps with any active immutable borrow
        check_overlap(&self.shared, &range)?;

        // Record a record of the mutable borrow
        self.owned.push(range);

        Ok(())
    }
}

fn is_disjoint(a: &Range<*const u8>, b: &Range<*const u8>) -> bool {
    b.start >= a.end || a.start >= b.end
}

fn check_overlap(
    existing: &[Range<*const u8>],
    range: &Range<*const u8>,
) -> Result<(), BorrowError> {
    if existing.iter().all(|i| is_disjoint(i, range)) {
        Ok(())
    } else {
        Err(BorrowError::new())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::error::Error;
    use std::mem;
    use std::slice;

    use super::{BorrowError, Ledger};

    // Super unsafe, but we only use it for testing `Ledger`
    fn unsafe_aliased_slice<T>(data: &mut [T]) -> &'static mut [T] {
        unsafe { slice::from_raw_parts_mut(data.as_mut_ptr(), data.len()) }
    }

    #[test]
    fn test_overlapping_immutable_borrows() -> Result<(), Box<dyn Error>> {
        let ledger = RefCell::new(Ledger::default());
        let data = vec![0u8; 128];

        Ledger::try_borrow(&ledger, &data[0..10])?;
        Ledger::try_borrow(&ledger, &data[0..100])?;
        Ledger::try_borrow(&ledger, &data[20..])?;

        Ok(())
    }

    #[test]
    fn test_nonoverlapping_borrows() -> Result<(), Box<dyn Error>> {
        let ledger = RefCell::new(Ledger::default());
        let mut data = vec![0; 16];
        let (a, b) = data.split_at_mut(4);

        let _a = Ledger::try_borrow_mut(&ledger, a)?;
        let _b = Ledger::try_borrow(&ledger, b)?;

        Ok(())
    }

    #[test]
    fn test_overlapping_borrows() -> Result<(), Box<dyn Error>> {
        let ledger = RefCell::new(Ledger::default());
        let mut data = vec![0; 16];
        let a = unsafe_aliased_slice(&mut data[4..8]);
        let b = unsafe_aliased_slice(&mut data[6..12]);
        let ab = Ledger::try_borrow(&ledger, a)?;

        // Should fail because it overlaps
        assert_eq!(
            Ledger::try_borrow_mut(&ledger, b).unwrap_err(),
            BorrowError::new(),
        );

        // Drop the first borrow
        mem::drop(ab);

        // Should succeed because previous borrow was dropped
        let bb = Ledger::try_borrow_mut(&ledger, b)?;

        // Should fail because it overlaps
        assert_eq!(
            Ledger::try_borrow(&ledger, a).unwrap_err(),
            BorrowError::new(),
        );

        // Drop the second borrow
        mem::drop(bb);

        // Should succeed because previous borrow was dropped
        let _ab = Ledger::try_borrow(&ledger, a)?;

        Ok(())
    }
}
