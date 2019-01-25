//! Types and traits representing binary JavaScript data.

use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use std::slice;
use context::{Context, Lock};
use borrow::{Borrow, BorrowMut, Ref, RefMut, LoanError};
use borrow::internal::Pointer;
use types::{Object, Managed, Value, build};
use types::internal::ValueInternal;
use result::NeonResult;
use neon_runtime;
use neon_runtime::raw;

/// The Node [`Buffer`](https://nodejs.org/api/buffer.html) type.
#[repr(C)]
pub struct JsBuffer(raw::Persistent);

impl JsBuffer {

    /// Constructs a new `Buffer` object, safely zero-filled.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, size: u32) -> NeonResult<&'a JsBuffer> {
        let isolate = cx.isolate().to_raw();
        build(cx, |out| {
            unsafe { neon_runtime::buffer::init_safe(out, isolate, size) }
        })
    }

    /// Constructs a new `Buffer` object, unsafely filled with uninitialized data.
    pub unsafe fn uninitialized<'a, C: Context<'a>>(cx: &mut C, size: u32) -> NeonResult<&'a JsBuffer> {
        let isolate = cx.isolate().to_raw();
        build(cx, |out| {
            unsafe { neon_runtime::buffer::init_unsafe(out, isolate, size) }
        })
    }

}

impl Managed for JsBuffer { }

impl ValueInternal for JsBuffer {
    fn name() -> String { "Buffer".to_string() }

    fn is_typeof<Other: Value>(other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_buffer(other.to_raw()) }
    }
}

impl Value for JsBuffer { }

impl Object for JsBuffer { }

/// The standard JS [`ArrayBuffer`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer) type.
#[repr(C)]
pub struct JsArrayBuffer(raw::Persistent);

impl JsArrayBuffer {

    /// Constructs a new `ArrayBuffer` object with the given size, in bytes.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, size: u32) -> NeonResult<&'a JsArrayBuffer> {
        let isolate = { cx.isolate().to_raw() };
        build(cx, |out| {
            unsafe { neon_runtime::arraybuffer::init(out, isolate, size) }
        })
    }

}

impl Managed for JsArrayBuffer { }

impl ValueInternal for JsArrayBuffer {
    fn name() -> String { "ArrayBuffer".to_string() }

    fn is_typeof<Other: Value>(other: &Other) -> bool {
        unsafe { neon_runtime::tag::is_arraybuffer(other.to_raw()) }
    }
}

impl Value for JsArrayBuffer { }

impl Object for JsArrayBuffer { }

/// A reference to the internal backing buffer data of a `Buffer` or `ArrayBuffer` object, which can be accessed via the `Borrow` and `BorrowMut` traits.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct BinaryData<'a> {
    base: *mut c_void,
    size: usize,
    phantom: PhantomData<&'a ()>
}

unsafe impl<'a> Pointer for BinaryData<'a> {
    unsafe fn as_ptr(&self) -> *const c_void {
        self.base
    }

    unsafe fn as_mut(&mut self) -> *mut c_void {
        self.base
    }
}

/// The trait for element types by which a buffer's binary data can be indexed.
pub trait BinaryViewType: Sized { }

impl BinaryViewType for u8 { }
impl BinaryViewType for i8 { }
impl BinaryViewType for u16 { }
impl BinaryViewType for i16 { }
impl BinaryViewType for u32 { }
impl BinaryViewType for i32 { }
impl BinaryViewType for u64 { }
impl BinaryViewType for i64 { }
impl BinaryViewType for f32 { }
impl BinaryViewType for f64 { }

impl<'a> BinaryData<'a> {

    /// Produces an immutable slice as a view into the contents of this buffer.
    /// 
    /// # Example:
    /// 
    /// ```no_run
    /// # use neon::prelude::*;
    /// # fn get_x_and_y(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// let b: Handle<JsArrayBuffer> = cx.argument(0)?;
    /// let (x, y) = cx.borrow(&b, |data| {
    ///     let slice = data.as_slice::<i32>();
    ///     (slice[0], slice[1])
    /// });
    /// # println!("({}, {})", x, y);
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    pub fn as_slice<T: BinaryViewType>(self) -> &'a [T] {
        let base = unsafe { mem::transmute(self.base) };
        let len = self.size / mem::size_of::<T>();
        unsafe { slice::from_raw_parts(base, len) }
    }

    /// Produces a mutable slice as a view into the contents of this buffer.
    /// 
    /// # Example:
    /// 
    /// ```no_run
    /// # use neon::prelude::*;
    /// # fn modify_buffer(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// let mut b: Handle<JsArrayBuffer> = cx.argument(0)?;
    /// cx.borrow_mut(&mut b, |data| {
    ///     let slice = data.as_mut_slice::<f64>();
    ///     slice[0] /= 2.0;
    ///     slice[1] *= 2.0;
    /// });
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    pub fn as_mut_slice<T: BinaryViewType>(self) -> &'a mut [T] {
        let base = unsafe { mem::transmute(self.base) };
        let len = self.size / mem::size_of::<T>();
        unsafe { slice::from_raw_parts_mut(base, len) }
    }

    /// Produces the length of the buffer, in bytes.
    pub fn len(self) -> usize {
        self.size
    }
}

impl<'a> Borrow for &'a JsBuffer {
    type Target = BinaryData<'a>;

    fn try_borrow<'b>(self, guard: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        let mut pointer: BinaryData = unsafe { mem::uninitialized() };
        unsafe {
            neon_runtime::buffer::data(&mut pointer.base, &mut pointer.size, self.to_raw());
            Ref::new(guard, pointer)
        }
    }
}

impl<'a> BorrowMut for &'a JsBuffer {
    fn try_borrow_mut<'b>(self, guard: &'b Lock<'b>) -> Result<RefMut<'b, Self::Target>, LoanError> {
        let mut pointer: BinaryData = unsafe { mem::uninitialized() };
        unsafe {
            neon_runtime::buffer::data(&mut pointer.base, &mut pointer.size, self.to_raw());
            RefMut::new(guard, pointer)
        }
    }
}

impl<'a> Borrow for &'a JsArrayBuffer {
    type Target = BinaryData<'a>;

    fn try_borrow<'b>(self, guard: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        let mut pointer: BinaryData = unsafe { mem::uninitialized() };
        unsafe {
            neon_runtime::arraybuffer::data(&mut pointer.base, &mut pointer.size, self.to_raw());
            Ref::new(guard, pointer)
        }
    }
}

impl<'a> BorrowMut for &'a JsArrayBuffer {
    fn try_borrow_mut<'b>(self, guard: &'b Lock<'b>) -> Result<RefMut<'b, Self::Target>, LoanError> {
        let mut pointer: BinaryData = unsafe { mem::uninitialized() };
        unsafe {
            neon_runtime::arraybuffer::data(&mut pointer.base, &mut pointer.size, self.to_raw());
            RefMut::new(guard, pointer)
        }
    }
}
