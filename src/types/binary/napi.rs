use std::cell::RefCell;
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Range};
use std::slice;

use neon_runtime::{raw, TypedArrayType};

use crate::context::{internal::Env, Context};
use crate::handle::{Handle, Managed};
use crate::result::{JsResult, NeonResult, ResultExt, Throw};
use crate::types::{internal::ValueInternal, Object, Value};

/// A trait for borrowing binary data from JavaScript values
///
/// Provides both statically and dynamically checked borrowing. Mutable borrows
/// are guaranteed not to overlap with other borrows.
pub trait Borrow: private::Sealed {
    type Item;

    /// Statically checked immutable borrow of binary data.
    ///
    /// This may not be used if a mutable borrow is in scope. For the dynamically
    /// checked variant see [`Borrow::try_borrow`].
    fn as_slice<'a: 'b, 'b, C>(&'b self, cx: &'b C) -> &'b [Self::Item]
    where
        C: Context<'a>;

    /// Statically checked mutable borrow of binary data.
    ///
    /// This may not be used if any other borrow is in scope. For the dynamically
    /// checked variant see [`Borrow::try_borrow_mut`].
    fn as_mut_slice<'a: 'b, 'b, C>(&'b mut self, cx: &'b mut C) -> &'b mut [Self::Item]
    where
        C: Context<'a>;

    /// Dynamically checked immutable borrow of binary data, returning an error if the
    /// the borrow would overlap with a mutable borrow.
    ///
    /// The borrow lasts until [`Ref`] exits scope.
    ///
    /// This is the dynamically checked version of [`Borrow::as_slice`].
    fn try_borrow<'a: 'b, 'b, C>(
        &self,
        lock: &'b Lock<'b, C>,
    ) -> Result<Ref<'b, Self::Item>, BorrowError>
    where
        C: Context<'a>;

    /// Dynamically checked mutable borrow of binary data, returning an error if the
    /// the borrow would overlap with an active borrow.
    ///
    /// The borrow lasts until [`RefMut`] exits scope.
    ///
    /// This is the dynamically checked version of [`Borrow::as_mut_slice`].
    fn try_borrow_mut<'a: 'b, 'b, C>(
        &mut self,
        lock: &'b Lock<'b, C>,
    ) -> Result<RefMut<'b, Self::Item>, BorrowError>
    where
        C: Context<'a>;
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
/// An error returned by [`Borrow::try_borrow`] or [`Borrow::try_borrow_mut`] indicating
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

#[derive(Debug)]
/// A temporary lock of an execution context.
///
/// While a lock is alive, no JavaScript code can be executed in the execution context.
///
/// Values that support the `Borrow` trait may be dynamically borrowed by passing a
/// [`Lock`].
pub struct Lock<'cx, C> {
    cx: &'cx C,
    ledger: RefCell<Ledger>,
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

    /// Dynamically checked immutable borrow.
    ///
    /// See [`Borrow::try_borrow`].
    pub fn try_borrow<T>(&self, buf: &T) -> Result<Ref<T::Item>, BorrowError>
    where
        T: Borrow,
    {
        buf.try_borrow(self)
    }

    /// Dynamically checked mutable borrow.
    ///
    /// See [`Borrow::try_borrow_mut`].
    pub fn try_borrow_mut<T>(&self, buf: &mut T) -> Result<RefMut<T::Item>, BorrowError>
    where
        T: Borrow,
    {
        buf.try_borrow_mut(self)
    }
}

#[derive(Debug, Default)]
// Bookkeeping for dynamically check borrowing rules
//
// Ranges are open on the end: `[start, end)`
struct Ledger {
    // Mutable borrows. Should never overlap with other borrows.
    owned: Vec<Range<*const u8>>,

    // Immutable borrows. May overlap or contain duplicates.
    shared: Vec<Range<*const u8>>,
}

impl Ledger {
    // Convert a slice of arbitrary type and size to a range of bytes addresses
    //
    // Alignment does not matter because we are only interested in bytes.
    fn slice_to_range<T>(data: &[T]) -> Range<*const u8> {
        let Range { start, end } = data.as_ptr_range();

        (start.cast())..(end.cast())
    }

    // Dynamically check a slice conforms to borrow rules before returning by
    // using interior mutability of the ledger.
    fn try_borrow<'a, T>(
        ledger: &'a RefCell<Self>,
        data: &'a [T],
    ) -> Result<Ref<'a, T>, BorrowError> {
        ledger.borrow_mut().try_add_borrow(data)?;

        Ok(Ref { ledger, data })
    }

    // Dynamically check a mutable slice conforms to borrow rules before returning by
    // using interior mutability of the ledger.
    fn try_borrow_mut<'a, T>(
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
        for borrow in self.owned.iter() {
            if borrow.start < range.end && range.start < borrow.end {
                return Err(BorrowError::new());
            }
        }

        // Record a record of the immutable borrow
        self.shared.push(range);

        Ok(())
    }

    // Try to add a mutable borrow to the ledger
    fn try_add_borrow_mut<T>(&mut self, data: &mut [T]) -> Result<(), BorrowError> {
        let range = Self::slice_to_range(data);

        // Check if the borrow overlaps with any active mutable borrow
        for borrow in self.owned.iter() {
            if borrow.start < range.end && range.start < borrow.end {
                return Err(BorrowError::new());
            }
        }

        // Check if the borrow overlaps with any active immutable borrow
        for borrow in self.shared.iter() {
            if borrow.start < range.end && range.start < borrow.end {
                return Err(BorrowError::new());
            }
        }

        // Record a record of the mutable borrow
        self.owned.push(range);

        Ok(())
    }
}

/// The Node [`Buffer`](https://nodejs.org/api/buffer.html) type.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JsBuffer(raw::Local);

impl JsBuffer {
    /// Constructs a new `Buffer` object, safely zero-filled.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, len: usize) -> JsResult<'a, Self> {
        let result = unsafe { neon_runtime::buffer::new(cx.env().to_raw(), len) };

        if let Ok(buf) = result {
            Ok(Handle::new_internal(Self(buf)))
        } else {
            Err(Throw)
        }
    }

    /// Constructs a new `Buffer` object with uninitialized memory
    pub unsafe fn uninitialized<'a, C: Context<'a>>(cx: &mut C, len: usize) -> JsResult<'a, Self> {
        let result = neon_runtime::buffer::uninitialized(cx.env().to_raw(), len);

        if let Ok((buf, _)) = result {
            Ok(Handle::new_internal(Self(buf)))
        } else {
            Err(Throw)
        }
    }

    /// Construct a new `Buffer` from bytes allocated by Rust
    pub fn external<'a, C, T>(cx: &mut C, data: T) -> Handle<'a, Self>
    where
        C: Context<'a>,
        T: AsMut<[u8]> + Send,
    {
        let env = cx.env().to_raw();
        let value = unsafe { neon_runtime::buffer::new_external(env, data) };

        Handle::new_internal(Self(value))
    }
}

impl Managed for JsBuffer {
    fn to_raw(self) -> raw::Local {
        self.0
    }

    fn from_raw(_env: Env, h: raw::Local) -> Self {
        Self(h)
    }
}

impl ValueInternal for JsBuffer {
    fn name() -> String {
        "Buffer".to_string()
    }

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool {
        unsafe { neon_runtime::tag::is_buffer(env.to_raw(), other.to_raw()) }
    }
}

impl Value for JsBuffer {}

impl Object for JsBuffer {}

impl private::Sealed for JsBuffer {}

impl Borrow for JsBuffer {
    type Item = u8;

    fn as_slice<'a: 'b, 'b, C>(&'b self, cx: &'b C) -> &'b [Self::Item]
    where
        C: Context<'a>,
    {
        unsafe { neon_runtime::buffer::as_mut_slice(cx.env().to_raw(), self.to_raw()) }
    }

    fn as_mut_slice<'a: 'b, 'b, C>(&'b mut self, cx: &'b mut C) -> &'b mut [Self::Item]
    where
        C: Context<'a>,
    {
        unsafe { neon_runtime::buffer::as_mut_slice(cx.env().to_raw(), self.to_raw()) }
    }

    fn try_borrow<'a: 'b, 'b, C>(
        &self,
        lock: &'b Lock<'b, C>,
    ) -> Result<Ref<'b, Self::Item>, BorrowError>
    where
        C: Context<'a>,
    {
        // The borrowed data must be guarded by `Ledger` before returning
        Ledger::try_borrow(&lock.ledger, unsafe {
            neon_runtime::buffer::as_mut_slice(lock.cx.env().to_raw(), self.to_raw())
        })
    }

    fn try_borrow_mut<'a: 'b, 'b, C>(
        &mut self,
        lock: &'b Lock<'b, C>,
    ) -> Result<RefMut<'b, Self::Item>, BorrowError>
    where
        C: Context<'a>,
    {
        // The borrowed data must be guarded by `Ledger` before returning
        Ledger::try_borrow_mut(&lock.ledger, unsafe {
            neon_runtime::buffer::as_mut_slice(lock.cx.env().to_raw(), self.to_raw())
        })
    }
}

/// The standard JS [`ArrayBuffer`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer) type.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JsArrayBuffer(raw::Local);

impl JsArrayBuffer {
    /// Constructs a new `JsArrayBuffer` object, safely zero-filled.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, len: usize) -> JsResult<'a, Self> {
        let result = unsafe { neon_runtime::arraybuffer::new(cx.env().to_raw(), len) };

        if let Ok(buf) = result {
            Ok(Handle::new_internal(Self(buf)))
        } else {
            Err(Throw)
        }
    }

    /// Construct a new `JsArrayBuffer` from bytes allocated by Rust
    pub fn external<'a, C, T>(cx: &mut C, data: T) -> Handle<'a, Self>
    where
        C: Context<'a>,
        T: AsMut<[u8]> + Send,
    {
        let env = cx.env().to_raw();
        let value = unsafe { neon_runtime::arraybuffer::new_external(env, data) };

        Handle::new_internal(JsArrayBuffer(value))
    }
}

impl Managed for JsArrayBuffer {
    fn to_raw(self) -> raw::Local {
        self.0
    }

    fn from_raw(_env: Env, h: raw::Local) -> Self {
        Self(h)
    }
}

impl ValueInternal for JsArrayBuffer {
    fn name() -> String {
        "JsArrayBuffer".to_string()
    }

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool {
        unsafe { neon_runtime::tag::is_arraybuffer(env.to_raw(), other.to_raw()) }
    }
}

impl Value for JsArrayBuffer {}

impl Object for JsArrayBuffer {}

impl private::Sealed for JsArrayBuffer {}

impl Borrow for JsArrayBuffer {
    type Item = u8;

    fn as_slice<'a: 'b, 'b, C>(&'b self, cx: &'b C) -> &'b [Self::Item]
    where
        C: Context<'a>,
    {
        unsafe { neon_runtime::arraybuffer::as_mut_slice(cx.env().to_raw(), self.to_raw()) }
    }

    fn as_mut_slice<'a: 'b, 'b, C>(&'b mut self, cx: &'b mut C) -> &'b mut [Self::Item]
    where
        C: Context<'a>,
    {
        unsafe { neon_runtime::arraybuffer::as_mut_slice(cx.env().to_raw(), self.to_raw()) }
    }

    fn try_borrow<'a: 'b, 'b, C>(
        &self,
        lock: &'b Lock<'b, C>,
    ) -> Result<Ref<'b, Self::Item>, BorrowError>
    where
        C: Context<'a>,
    {
        // The borrowed data must be guarded by `Ledger` before returning
        Ledger::try_borrow(&lock.ledger, unsafe {
            neon_runtime::arraybuffer::as_mut_slice(lock.cx.env().to_raw(), self.to_raw())
        })
    }

    fn try_borrow_mut<'a: 'b, 'b, C>(
        &mut self,
        lock: &'b Lock<'b, C>,
    ) -> Result<RefMut<'b, Self::Item>, BorrowError>
    where
        C: Context<'a>,
    {
        // The borrowed data must be guarded by `Ledger` before returning
        Ledger::try_borrow_mut(&lock.ledger, unsafe {
            neon_runtime::arraybuffer::as_mut_slice(lock.cx.env().to_raw(), self.to_raw())
        })
    }
}

/// The standard JS [`TypedArray`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/TypedArray) type.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JsTypedArray<T> {
    local: raw::Local,
    _type: PhantomData<T>,
}

impl<T> private::Sealed for JsTypedArray<T> {}

impl<T: Copy> Managed for JsTypedArray<T> {
    fn to_raw(self) -> raw::Local {
        self.local
    }

    fn from_raw(_env: Env, local: raw::Local) -> Self {
        Self {
            local,
            _type: PhantomData,
        }
    }
}

impl<T: Copy> Borrow for JsTypedArray<T> {
    type Item = T;

    fn as_slice<'a: 'b, 'b, C>(&'b self, cx: &'b C) -> &'b [Self::Item]
    where
        C: Context<'a>,
    {
        unsafe {
            let env = cx.env().to_raw();
            let value = self.to_raw();
            let info = neon_runtime::typedarray::info(env, value);

            slice::from_raw_parts(info.data.cast(), info.length)
        }
    }

    fn as_mut_slice<'a: 'b, 'b, C>(&'b mut self, cx: &'b mut C) -> &'b mut [Self::Item]
    where
        C: Context<'a>,
    {
        unsafe {
            let env = cx.env().to_raw();
            let value = self.to_raw();
            let info = neon_runtime::typedarray::info(env, value);

            slice::from_raw_parts_mut(info.data.cast(), info.length)
        }
    }

    fn try_borrow<'a: 'b, 'b, C>(
        &self,
        lock: &'b Lock<'b, C>,
    ) -> Result<Ref<'b, Self::Item>, BorrowError>
    where
        C: Context<'a>,
    {
        unsafe {
            let env = lock.cx.env().to_raw();
            let value = self.to_raw();
            let info = neon_runtime::typedarray::info(env, value);

            // The borrowed data must be guarded by `Ledger` before returning
            Ledger::try_borrow(
                &lock.ledger,
                slice::from_raw_parts(info.data.cast(), info.length),
            )
        }
    }

    fn try_borrow_mut<'a: 'b, 'b, C>(
        &mut self,
        lock: &'b Lock<'b, C>,
    ) -> Result<RefMut<'b, Self::Item>, BorrowError>
    where
        C: Context<'a>,
    {
        unsafe {
            let env = lock.cx.env().to_raw();
            let value = self.to_raw();
            let info = neon_runtime::typedarray::info(env, value);

            // The borrowed data must be guarded by `Ledger` before returning
            Ledger::try_borrow_mut(
                &lock.ledger,
                slice::from_raw_parts_mut(info.data.cast(), info.length),
            )
        }
    }
}

macro_rules! impl_typed_array {
    ($name:expr, $typ:ty, $($pattern:pat)|+$(,)?) => {
        impl Value for JsTypedArray<$typ> {}

        impl Object for JsTypedArray<$typ> {}

        impl ValueInternal for JsTypedArray<$typ> {
            fn name() -> String {
                $name.to_string()
            }

            fn is_typeof<Other: Value>(env: Env, other: Other) -> bool {
                let env = env.to_raw();
                let other = other.to_raw();

                if unsafe { !neon_runtime::tag::is_typedarray(env, other) } {
                    return false;
                }

                let info = unsafe { neon_runtime::typedarray::info(env, other) };

                matches!(info.typ, $($pattern)|+)
            }
        }
    };
}

impl_typed_array!("Int8Array", i8, TypedArrayType::I8);
impl_typed_array!(
    "Uint8Array",
    u8,
    TypedArrayType::U8 | TypedArrayType::U8Clamped,
);
impl_typed_array!("Int16Array", i16, TypedArrayType::I16);
impl_typed_array!("Uint16Array", u16, TypedArrayType::U16);
impl_typed_array!("Int32Array", i32, TypedArrayType::I32);
impl_typed_array!("Uint32Array", u32, TypedArrayType::U32);
impl_typed_array!("Float32Array", f32, TypedArrayType::F32);
impl_typed_array!("Float64Array", f64, TypedArrayType::F64);
impl_typed_array!("BigInt64Array", i64, TypedArrayType::I64);
impl_typed_array!("BigUint64Array", u64, TypedArrayType::U64);

mod private {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::error::Error;
    use std::mem;
    use std::slice;

    use super::{BorrowError, BorrowError, Ledger};

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
