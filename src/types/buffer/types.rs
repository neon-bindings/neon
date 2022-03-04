use std::marker::PhantomData;
use std::slice;

use neon_runtime::{raw, TypedArrayType};

use crate::context::{internal::Env, Context};
use crate::handle::{internal::TransparentNoCopyWrapper, Handle, Managed};
use crate::result::{JsResult, Throw};
use crate::types::{private::ValueInternal, Object, Value};

use super::lock::{Ledger, Lock};
use super::{private, BorrowError, Ref, RefMut, TypedArray};

/// The Node [`Buffer`](https://nodejs.org/api/buffer.html) type.
#[derive(Debug)]
#[repr(transparent)]
pub struct JsBuffer(raw::Local);

impl JsBuffer {
    /// Constructs a new `Buffer` object, safely zero-filled.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, len: usize) -> JsResult<'a, Self> {
        let result = unsafe { neon_runtime::buffer::new(cx.env().to_raw(), len) };

        if let Ok(buf) = result {
            Ok(Handle::new_internal(Self(buf)))
        } else {
            Err(Throw::new())
        }
    }

    /// Constructs a new `Buffer` object with uninitialized memory
    pub unsafe fn uninitialized<'a, C: Context<'a>>(cx: &mut C, len: usize) -> JsResult<'a, Self> {
        let result = neon_runtime::buffer::uninitialized(cx.env().to_raw(), len);

        if let Ok((buf, _)) = result {
            Ok(Handle::new_internal(Self(buf)))
        } else {
            Err(Throw::new())
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

unsafe impl TransparentNoCopyWrapper for JsBuffer {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl Managed for JsBuffer {
    fn to_raw(&self) -> raw::Local {
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

    fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_buffer(env.to_raw(), other.to_raw()) }
    }
}

impl Value for JsBuffer {}

impl Object for JsBuffer {}

impl private::Sealed for JsBuffer {}

impl TypedArray for JsBuffer {
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
#[derive(Debug)]
#[repr(transparent)]
pub struct JsArrayBuffer(raw::Local);

impl JsArrayBuffer {
    /// Constructs a new `JsArrayBuffer` object, safely zero-filled.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, len: usize) -> JsResult<'a, Self> {
        let result = unsafe { neon_runtime::arraybuffer::new(cx.env().to_raw(), len) };

        if let Ok(buf) = result {
            Ok(Handle::new_internal(Self(buf)))
        } else {
            Err(Throw::new())
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

        Handle::new_internal(Self(value))
    }
}

unsafe impl TransparentNoCopyWrapper for JsArrayBuffer {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl Managed for JsArrayBuffer {
    fn to_raw(&self) -> raw::Local {
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

    fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_arraybuffer(env.to_raw(), other.to_raw()) }
    }
}

impl Value for JsArrayBuffer {}

impl Object for JsArrayBuffer {}

impl private::Sealed for JsArrayBuffer {}

impl TypedArray for JsArrayBuffer {
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
#[derive(Debug)]
#[repr(transparent)]
pub struct JsTypedArray<T> {
    local: raw::Local,
    _type: PhantomData<T>,
}

impl<T> private::Sealed for JsTypedArray<T> {}

unsafe impl<T> TransparentNoCopyWrapper for JsTypedArray<T> {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.local
    }
}

impl<T> Managed for JsTypedArray<T> {
    fn to_raw(&self) -> raw::Local {
        self.local
    }

    fn from_raw(_env: Env, local: raw::Local) -> Self {
        Self {
            local,
            _type: PhantomData,
        }
    }
}

impl<T: Copy> TypedArray for JsTypedArray<T> {
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

            fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool {
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
