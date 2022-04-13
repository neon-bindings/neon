use std::{fmt::Debug, mem};

use crate::types::Value;

pub trait SuperType<T: Value> {
    fn upcast_internal(v: &T) -> Self;
}

#[doc(hidden)]
/// Trait asserting that `Self` is a transparent wrapper around `Self::Inner`
/// with identical representation and may be safely transmuted.
///
/// # Safety
/// `Self` must be `#[repr(transparent)]` with a field `Self::Inner`
pub unsafe trait TransparentNoCopyWrapper: Sized {
    type Inner: Debug + Copy;

    // A default implementation cannot be provided because it would create
    // dependently sized types. This may be supported in a future Rust version.
    fn into_inner(self) -> Self::Inner;

    fn wrap_ref(s: &Self::Inner) -> &Self {
        unsafe { mem::transmute(s) }
    }

    fn wrap_mut(s: &mut Self::Inner) -> &mut Self {
        unsafe { mem::transmute(s) }
    }
}
