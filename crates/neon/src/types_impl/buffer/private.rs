use crate::sys::TypedArrayType;

pub trait Sealed {}

/// A marker trait for all possible element types of binary buffers.
pub trait Binary: Copy {
    /// The internal Node-API enum value for this binary type.
    const TYPE_TAG: TypedArrayType;
}
