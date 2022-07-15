use std::{marker::PhantomData, slice};

use crate::{
    context::{internal::Env, Context},
    handle::{internal::TransparentNoCopyWrapper, Handle, Managed},
    object::Object,
    result::{JsResult, Throw},
    sys::{self, raw, TypedArrayType},
    types::buffer::{
        lock::{Ledger, Lock},
        private::{self, JsTypedArrayInner},
        BorrowError, Ref, RefMut, TypedArray,
    },
    types::{private::ValueInternal, Value},
};

use doc_comment::doc_comment;

/// The Node [`Buffer`](https://nodejs.org/api/buffer.html) type.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// use neon::types::buffer::TypedArray;
///
/// fn make_sequence(mut cx: FunctionContext) -> JsResult<JsBuffer> {
///     let len = cx.argument::<JsNumber>(0)?.value(&mut cx);
///     let mut buffer = cx.buffer(len as usize)?;
///
///     for (i, elem) in buffer.as_mut_slice(&mut cx).iter_mut().enumerate() {
///         *elem = i as u8;
///     }
///
///     Ok(buffer)
/// }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct JsBuffer(raw::Local);

impl JsBuffer {
    /// Constructs a new `Buffer` object, safely zero-filled.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, len: usize) -> JsResult<'a, Self> {
        let result = unsafe { sys::buffer::new(cx.env().to_raw(), len) };

        if let Ok(buf) = result {
            Ok(Handle::new_internal(Self(buf)))
        } else {
            Err(Throw::new())
        }
    }

    /// Constructs a new `Buffer` object with uninitialized memory
    pub unsafe fn uninitialized<'a, C: Context<'a>>(cx: &mut C, len: usize) -> JsResult<'a, Self> {
        let result = sys::buffer::uninitialized(cx.env().to_raw(), len);

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
        T: AsMut<[u8]> + Send + 'static,
    {
        let env = cx.env().to_raw();
        let value = unsafe { sys::buffer::new_external(env, data) };

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
        unsafe { sys::tag::is_buffer(env.to_raw(), other.to_raw()) }
    }
}

impl Value for JsBuffer {}

impl Object for JsBuffer {}

impl private::Sealed for JsBuffer {}

impl TypedArray for JsBuffer {
    type Item = u8;

    fn as_slice<'cx, 'a, C>(&self, cx: &'a C) -> &'a [Self::Item]
    where
        C: Context<'cx>,
    {
        // # Safety
        // Only the `Context` with the *most* narrow scope is accessible because `compute_scoped`
        // and `execute_scope` take an exclusive reference to `Context`. A handle is always
        // associated with a `Context` and the value will not be garbage collected while that
        // `Context` is in scope. This means that the referenced data is valid *at least* as long
        // as `Context`, even if the `Handle` is dropped.
        unsafe { sys::buffer::as_mut_slice(cx.env().to_raw(), self.to_raw()) }
    }

    fn as_mut_slice<'cx, 'a, C>(&mut self, cx: &'a mut C) -> &'a mut [Self::Item]
    where
        C: Context<'cx>,
    {
        // # Safety
        // See `as_slice`
        unsafe { sys::buffer::as_mut_slice(cx.env().to_raw(), self.to_raw()) }
    }

    fn try_borrow<'cx, 'a, C>(&self, lock: &'a Lock<C>) -> Result<Ref<'a, Self::Item>, BorrowError>
    where
        C: Context<'cx>,
    {
        // The borrowed data must be guarded by `Ledger` before returning
        Ledger::try_borrow(&lock.ledger, unsafe {
            sys::buffer::as_mut_slice(lock.cx.env().to_raw(), self.to_raw())
        })
    }

    fn try_borrow_mut<'cx, 'a, C>(
        &mut self,
        lock: &'a Lock<C>,
    ) -> Result<RefMut<'a, Self::Item>, BorrowError>
    where
        C: Context<'cx>,
    {
        // The borrowed data must be guarded by `Ledger` before returning
        Ledger::try_borrow_mut(&lock.ledger, unsafe {
            sys::buffer::as_mut_slice(lock.cx.env().to_raw(), self.to_raw())
        })
    }

    fn byte_length<'cx, C: Context<'cx>>(&self, cx: &mut C) -> usize {
        unsafe { sys::buffer::size(cx.env().to_raw(), self.to_raw()) }
    }
}

/// The standard JS [`ArrayBuffer`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer) type.
///
/// # Example
///
/// ```
/// # use neon::prelude::*;
/// use neon::types::buffer::TypedArray;
///
/// fn make_sequence(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
///     let len = cx.argument::<JsNumber>(0)?.value(&mut cx);
///     let mut buffer = cx.array_buffer(len as usize)?;
///
///     for (i, elem) in buffer.as_mut_slice(&mut cx).iter_mut().enumerate() {
///         *elem = i as u8;
///     }
///
///     Ok(buffer)
/// }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct JsArrayBuffer(raw::Local);

impl JsArrayBuffer {
    /// Constructs a new `JsArrayBuffer` object, safely zero-filled.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, len: usize) -> JsResult<'a, Self> {
        let result = unsafe { sys::arraybuffer::new(cx.env().to_raw(), len) };

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
        T: AsMut<[u8]> + Send + 'static,
    {
        let env = cx.env().to_raw();
        let value = unsafe { sys::arraybuffer::new_external(env, data) };

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
        unsafe { sys::tag::is_arraybuffer(env.to_raw(), other.to_raw()) }
    }
}

impl Value for JsArrayBuffer {}

impl Object for JsArrayBuffer {}

impl private::Sealed for JsArrayBuffer {}

impl TypedArray for JsArrayBuffer {
    type Item = u8;

    fn as_slice<'cx, 'a, C>(&self, cx: &'a C) -> &'a [Self::Item]
    where
        C: Context<'cx>,
    {
        unsafe { sys::arraybuffer::as_mut_slice(cx.env().to_raw(), self.to_raw()) }
    }

    fn as_mut_slice<'cx, 'a, C>(&mut self, cx: &'a mut C) -> &'a mut [Self::Item]
    where
        C: Context<'cx>,
    {
        unsafe { sys::arraybuffer::as_mut_slice(cx.env().to_raw(), self.to_raw()) }
    }

    fn try_borrow<'cx, 'a, C>(&self, lock: &'a Lock<C>) -> Result<Ref<'a, Self::Item>, BorrowError>
    where
        C: Context<'cx>,
    {
        // The borrowed data must be guarded by `Ledger` before returning
        Ledger::try_borrow(&lock.ledger, unsafe {
            sys::arraybuffer::as_mut_slice(lock.cx.env().to_raw(), self.to_raw())
        })
    }

    fn try_borrow_mut<'cx, 'a, C>(
        &mut self,
        lock: &'a Lock<C>,
    ) -> Result<RefMut<'a, Self::Item>, BorrowError>
    where
        C: Context<'cx>,
    {
        // The borrowed data must be guarded by `Ledger` before returning
        Ledger::try_borrow_mut(&lock.ledger, unsafe {
            sys::arraybuffer::as_mut_slice(lock.cx.env().to_raw(), self.to_raw())
        })
    }

    fn byte_length<'cx, C: Context<'cx>>(&self, cx: &mut C) -> usize {
        unsafe { sys::arraybuffer::size(cx.env().to_raw(), self.to_raw()) }
    }
}

/// A marker trait for all possible element types of binary buffers.
///
/// This trait can only be implemented within the Neon library.
pub trait Binary: private::Sealed + Copy + std::fmt::Debug {
    /// The internal Node-API enum value for this binary type.
    const TYPE_TAG: TypedArrayType;
}

impl<T: Binary> Copy for JsTypedArrayInner<T> {}

/// The family of JS [typed array][typed-arrays] types.
///
/// ## Typed Arrays
///
/// JavaScript's [typed arrays][typed-arrays] are objects that allow efficiently reading
/// and writing raw binary data in memory. In Neon, the generic type `JsTypedArray<T>`
/// represents a JavaScript typed array with element type `T`. For example, a JavaScript
/// [`Uint32Array`][Uint32Array] represents a compact array of 32-bit unsigned integers,
/// and is represented in Neon as a `JsTypedArray<u32>`.
///
/// Neon also offers a set of convenience shorthands for concrete instances of
/// `JsTypedArray`, named after their corresponding JavaScript type. For example,
/// `JsTypedArray<u32>` can also be referred to as [`JsUint32Array`][JsUint32Array].
///
/// The following table shows the complete set of typed array types, with both their
/// JavaScript and Neon types:
///
/// | Rust Type                      | Convenience Type                       | JavaScript Type                    |
/// | ------------------------------ | -------------------------------------- | ---------------------------------- |
/// | `JsTypedArray<`[`u8`][u8]`>`   | [`JsUint8Array`][JsUint8Array]         | [`Uint8Array`][Uint8Array]         |
/// | `JsTypedArray<`[`i8`][i8]`>`   | [`JsInt8Array`][JsInt8Array]           | [`Int8Array`][Int8Array]           |
/// | `JsTypedArray<`[`u16`][u16]`>` | [`JsUint16Array`][JsUint16Array]       | [`Uint16Array`][Uint16Array]       |
/// | `JsTypedArray<`[`i16`][i16]`>` | [`JsInt16Array`][JsInt16Array]         | [`Int16Array`][Int16Array]         |
/// | `JsTypedArray<`[`u32`][u32]`>` | [`JsUint32Array`][JsUint32Array]       | [`Uint32Array`][Uint32Array]       |
/// | `JsTypedArray<`[`i32`][i32]`>` | [`JsInt32Array`][JsInt32Array]         | [`Int32Array`][Int32Array]         |
/// | `JsTypedArray<`[`u64`][u64]`>` | [`JsBigUint64Array`][JsBigUint64Array] | [`BigUint64Array`][BigUint64Array] |
/// | `JsTypedArray<`[`i64`][i64]`>` | [`JsBigInt64Array`][JsBigInt64Array]   | [`BigInt64Array`][BigInt64Array]   |
/// | `JsTypedArray<`[`f32`][f32]`>` | [`JsFloat32Array`][JsFloat32Array]     | [`Float32Array`][Float32Array]     |
/// | `JsTypedArray<`[`f64`][f64]`>` | [`JsFloat64Array`][JsFloat64Array]     | [`Float64Array`][Float64Array]     |
///
/// ### Example: Creating an integer array
///
/// This example creates a typed array of unsigned 32-bit integers with a user-specified
/// length:
///
/// ```
/// # use neon::prelude::*;
/// fn create_int_array(mut cx: FunctionContext) -> JsResult<JsTypedArray<u32>> {
///     let len = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
///     JsTypedArray::new(&mut cx, len)
/// }
/// ```
///
/// ## Buffers
///
/// Typed arrays are managed with the [`ArrayBuffer`][ArrayBuffer] type, which controls
/// the storage of the underlying data buffer, and several typed views for managing access
/// to the buffer. Neon provides access to the `ArrayBuffer` class with the
/// [`JsArrayBuffer`](crate::types::JsArrayBuffer) type.
///
/// Node also provides a [`Buffer`][Buffer] type, which is built on top of `ArrayBuffer`
/// and provides additional functionality. Neon provides access to the `Buffer` class
/// with the [`JsBuffer`](crate::types::JsBuffer) type.
///
/// Many of Node's I/O APIs work with these types, and they can also be used for
/// compact in-memory data structures, which can be shared efficiently between
/// JavaScript and Rust without copying.
///
/// [u8]: std::primitive::u8
/// [i8]: std::primitive::i8
/// [u16]: std::primitive::u16
/// [i16]: std::primitive::i16
/// [u32]: std::primitive::u32
/// [i32]: std::primitive::i32
/// [u64]: std::primitive::u64
/// [i64]: std::primitive::i64
/// [f32]: std::primitive::f32
/// [f64]: std::primitive::f64
/// [JsUint8Array]: crate::types::JsUint8Array
/// [JsInt8Array]: crate::types::JsInt8Array
/// [JsUint16Array]: crate::types::JsUint16Array
/// [JsInt16Array]: crate::types::JsInt16Array
/// [JsUint32Array]: crate::types::JsUint32Array
/// [JsInt32Array]: crate::types::JsInt32Array
/// [JsBigUint64Array]: crate::types::JsBigUint64Array
/// [JsBigInt64Array]: crate::types::JsBigInt64Array
/// [JsFloat32Array]: crate::types::JsFloat32Array
/// [JsFloat64Array]: crate::types::JsFloat64Array
/// [Uint8Array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array
/// [Int8Array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Int8Array
/// [Uint16Array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint16Array
/// [Int16Array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Int16Array
/// [Uint32Array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint32Array
/// [Int32Array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Int32Array
/// [BigUint64Array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigUint64Array
/// [BigInt64Array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt64Array
/// [Float32Array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Float32Array
/// [Float64Array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Float64Array
/// [typed-arrays]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Typed_arrays
/// [ArrayBuffer]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer
/// [Buffer]: https://nodejs.org/api/buffer.html
#[derive(Debug)]
#[repr(transparent)]
pub struct JsTypedArray<T: Binary>(JsTypedArrayInner<T>);

impl<T: Binary> private::Sealed for JsTypedArray<T> {}

unsafe impl<T: Binary> TransparentNoCopyWrapper for JsTypedArray<T> {
    type Inner = JsTypedArrayInner<T>;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl<T: Binary> Managed for JsTypedArray<T> {
    fn to_raw(&self) -> raw::Local {
        self.0.local
    }

    // This method should be `unsafe`
    // https://github.com/neon-bindings/neon/issues/885
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn from_raw(env: Env, local: raw::Local) -> Self {
        // Safety: Recomputing this information ensures that the lifetime of the
        //         buffer handle matches the lifetime of the typed array handle.
        let info = unsafe { sys::typedarray::info(env.to_raw(), local) };

        Self(JsTypedArrayInner {
            local,
            buffer: info.buf,
            byte_offset: info.offset,
            len: info.length,
            _type: PhantomData,
        })
    }
}

impl<T: Binary> TypedArray for JsTypedArray<T> {
    type Item = T;

    fn as_slice<'cx, 'a, C>(&self, cx: &'a C) -> &'a [Self::Item]
    where
        C: Context<'cx>,
    {
        unsafe {
            let env = cx.env().to_raw();
            let value = self.to_raw();
            let info = sys::typedarray::info(env, value);

            slice::from_raw_parts(info.data.cast(), info.length)
        }
    }

    fn as_mut_slice<'cx, 'a, C>(&mut self, cx: &'a mut C) -> &'a mut [Self::Item]
    where
        C: Context<'cx>,
    {
        unsafe {
            let env = cx.env().to_raw();
            let value = self.to_raw();
            let info = sys::typedarray::info(env, value);

            slice::from_raw_parts_mut(info.data.cast(), info.length)
        }
    }

    fn try_borrow<'cx, 'b, C>(
        &self,
        lock: &'b Lock<'b, C>,
    ) -> Result<Ref<'b, Self::Item>, BorrowError>
    where
        C: Context<'cx>,
    {
        unsafe {
            let env = lock.cx.env().to_raw();
            let value = self.to_raw();
            let info = sys::typedarray::info(env, value);

            // The borrowed data must be guarded by `Ledger` before returning
            Ledger::try_borrow(
                &lock.ledger,
                slice::from_raw_parts(info.data.cast(), info.length),
            )
        }
    }

    fn try_borrow_mut<'cx, 'a, C>(
        &mut self,
        lock: &'a Lock<'a, C>,
    ) -> Result<RefMut<'a, Self::Item>, BorrowError>
    where
        C: Context<'cx>,
    {
        unsafe {
            let env = lock.cx.env().to_raw();
            let value = self.to_raw();
            let info = sys::typedarray::info(env, value);

            // The borrowed data must be guarded by `Ledger` before returning
            Ledger::try_borrow_mut(
                &lock.ledger,
                slice::from_raw_parts_mut(info.data.cast(), info.length),
            )
        }
    }

    fn byte_length<'cx, C: Context<'cx>>(&self, cx: &mut C) -> usize {
        self.len(cx) * std::mem::size_of::<Self::Item>()
    }
}

impl<T: Binary> JsTypedArray<T> {
    /// Constructs a typed array that views `buffer`.
    ///
    /// The resulting typed array has `(buffer.byte_length() / size_of::<T>())` elements.
    pub fn from_buffer<'cx, C>(cx: &mut C, buffer: Handle<JsArrayBuffer>) -> JsResult<'cx, Self>
    where
        C: Context<'cx>,
    {
        let byte_length = buffer.byte_length(cx);
        let elt_size = std::mem::size_of::<T>();
        let len = byte_length / elt_size;

        if (len * elt_size) != byte_length {
            panic!(
                "byte length of typed array should be a multiple of {}",
                elt_size
            );
        }

        Self::from_buffer_region(cx, buffer, 0, len)
    }

    /// Constructs a typed array that views a region of `buffer` starting
    /// at the specified byte offset and with the specified number of elements.
    ///
    /// The resulting typed array has `len` elements and byte length
    /// `(len * size_of::<T>())`.
    pub fn from_buffer_region<'cx, C>(
        cx: &mut C,
        buffer: Handle<JsArrayBuffer>,
        byte_offset: usize,
        len: usize,
    ) -> JsResult<'cx, Self>
    where
        C: Context<'cx>,
    {
        let result = unsafe {
            sys::typedarray::new(
                cx.env().to_raw(),
                T::TYPE_TAG,
                buffer.to_raw(),
                byte_offset,
                len,
            )
        };

        if let Ok(arr) = result {
            Ok(Handle::new_internal(Self(JsTypedArrayInner {
                local: arr,
                buffer: buffer.to_raw(),
                byte_offset,
                len,
                _type: PhantomData,
            })))
        } else {
            Err(Throw::new())
        }
    }

    /// Constructs a new typed array of length `len`.
    ///
    /// The resulting typed array has a newly allocated storage buffer of
    /// size `(len * size_of::<T>())` bytes.
    pub fn new<'cx, C>(cx: &mut C, len: usize) -> JsResult<'cx, Self>
    where
        C: Context<'cx>,
    {
        let buffer = cx.array_buffer(len * std::mem::size_of::<T>())?;
        Self::from_buffer_region(cx, buffer, 0, len)
    }

    /// Returns the [`JsArrayBuffer`](JsArrayBuffer) that owns the underlying storage buffer
    /// for this typed array.
    ///
    /// Note that the typed array might only reference a region of the buffer; use the
    /// [`byte_offset()`](JsTypedArray::byte_offset) and
    /// [`byte_length()`](crate::types::buffer::TypedArray::byte_length) methods to
    /// determine the region.
    pub fn buffer<'cx, C>(&self, cx: &mut C) -> Handle<'cx, JsArrayBuffer>
    where
        C: Context<'cx>,
    {
        Handle::new_internal(JsArrayBuffer::from_raw(cx.env(), self.0.buffer))
    }

    /// Returns the offset (in bytes) of the typed array from the start of its
    /// [`JsArrayBuffer`](JsArrayBuffer).
    pub fn byte_offset<'cx, C>(&self, _cx: &mut C) -> usize
    where
        C: Context<'cx>,
    {
        self.0.byte_offset
    }

    /// Returns the length of the typed array, i.e. the number of elements.
    ///
    /// Note that, depending on the element size, this is not necessarily the same as
    /// [`byte_length()`](crate::types::buffer::TypedArray::byte_length). In particular:
    ///
    /// ```ignore
    /// self.byte_length() == self.len() * size_of::<T>()
    /// ```
    #[allow(clippy::len_without_is_empty)]
    pub fn len<'cx, C>(&self, _cx: &mut C) -> usize
    where
        C: Context<'cx>,
    {
        self.0.len
    }
}

macro_rules! impl_typed_array {
    ($typ:ident, $etyp:ty, $($pattern:pat)|+, $tag:ident, $alias:ident, $two:expr$(,)?) => {
        impl private::Sealed for $etyp {}

        impl Binary for $etyp {
            const TYPE_TAG: TypedArrayType = TypedArrayType::$tag;
        }

        impl Value for JsTypedArray<$etyp> {}

        impl Object for JsTypedArray<$etyp> {}

        impl ValueInternal for JsTypedArray<$etyp> {
            fn name() -> String {
                stringify!($typ).to_string()
            }

            fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool {
                let env = env.to_raw();
                let other = other.to_raw();

                if unsafe { !sys::tag::is_typedarray(env, other) } {
                    return false;
                }

                let info = unsafe { sys::typedarray::info(env, other) };

                matches!(info.typ, $($pattern)|+)
            }
        }

        doc_comment! {
            concat!(
                "The standard JS [`",
                stringify!($typ),
                "`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/",
                stringify!($typ),
                ") type.

# Example

```
# use neon::prelude::*;
use neon::types::buffer::TypedArray;

fn double(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut array: Handle<",
                stringify!($alias),
                "> = cx.argument(0)?;

    for elem in array.as_mut_slice(&mut cx).iter_mut() {
        *elem *= ",
                stringify!($two),
                ";
    }

    Ok(cx.undefined())
}
```",
            ),
            pub type $alias = JsTypedArray<$etyp>;
        }
    };
}

impl_typed_array!(Int8Array, i8, TypedArrayType::I8, I8, JsInt8Array, 2);
impl_typed_array!(
    Uint8Array,
    u8,
    TypedArrayType::U8 | TypedArrayType::U8Clamped,
    U8,
    JsUint8Array,
    2,
);
impl_typed_array!(Int16Array, i16, TypedArrayType::I16, I16, JsInt16Array, 2);
impl_typed_array!(Uint16Array, u16, TypedArrayType::U16, U16, JsUint16Array, 2);
impl_typed_array!(Int32Array, i32, TypedArrayType::I32, I32, JsInt32Array, 2);
impl_typed_array!(Uint32Array, u32, TypedArrayType::U32, U32, JsUint32Array, 2);
impl_typed_array!(
    Float32Array,
    f32,
    TypedArrayType::F32,
    F32,
    JsFloat32Array,
    2.0,
);
impl_typed_array!(
    Float64Array,
    f64,
    TypedArrayType::F64,
    F64,
    JsFloat64Array,
    2.0,
);
impl_typed_array!(
    BigInt64Array,
    i64,
    TypedArrayType::I64,
    I64,
    JsBigInt64Array,
    2,
);
impl_typed_array!(
    BigUint64Array,
    u64,
    TypedArrayType::U64,
    U64,
    JsBigUint64Array,
    2,
);
