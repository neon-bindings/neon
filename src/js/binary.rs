//! Types and traits representing binary JavaScript data.

use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use std::slice;
use vm::{Context, JsResult};
use js::{Value, Object, Borrow, BorrowMut, Ref, RefMut, LoanError, build};
use js::internal::ValueInternal;
use mem::Managed;
use vm::VmGuard;
use vm::internal::Pointer;
use neon_runtime;
use neon_runtime::raw;

/// The Node [`Buffer`](https://nodejs.org/api/buffer.html) type.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsBuffer(raw::Local);

impl JsBuffer {

    /// Constructs a new `Buffer` object.
    pub fn new<'a, C: Context<'a>>(_: &mut C, size: u32) -> JsResult<'a, JsBuffer> {
        build(|out| { unsafe { neon_runtime::buffer::new(out, size) } })
    }

}

impl Managed for JsBuffer {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsBuffer(h) }
}

impl ValueInternal for JsBuffer {
    fn name() -> String { "Buffer".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_buffer(other.to_raw()) }
    }
}

impl Value for JsBuffer { }

impl Object for JsBuffer { }

/// The standard JS [`ArrayBuffer`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer) type.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsArrayBuffer(raw::Local);

impl JsArrayBuffer {

    /// Constructs a new `ArrayBuffer` object with the given size, in bytes.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, size: u32) -> JsResult<'a, JsArrayBuffer> {
        build(|out| { unsafe { neon_runtime::arraybuffer::new(out, mem::transmute(cx.isolate()), size) } })
    }

}

impl Managed for JsArrayBuffer {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsArrayBuffer(h) }
}

impl ValueInternal for JsArrayBuffer {
    fn name() -> String { "ArrayBuffer".to_string() }

    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_arraybuffer(other.to_raw()) }
    }
}

impl Value for JsArrayBuffer { }

impl Object for JsArrayBuffer { }

/// The internal backing buffer data of an `ArrayBuffer`, which can be access via the `Borrow` and `BorrowMut` traits.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ArrayBufferData<'a> {
    base: *mut c_void,
    size: usize,
    phantom: PhantomData<&'a ()>
}

unsafe impl<'a> Pointer for ArrayBufferData<'a> {
    unsafe fn as_ptr(&self) -> *const c_void {
        self.base
    }

    unsafe fn as_mut(&mut self) -> *mut c_void {
        self.base
    }
}

/// The trait for element types by which an `ArrayBuffer` can be indexed.
pub trait ArrayBufferViewType: Sized { }

impl ArrayBufferViewType for u8 { }
impl ArrayBufferViewType for i8 { }
impl ArrayBufferViewType for u16 { }
impl ArrayBufferViewType for i16 { }
impl ArrayBufferViewType for u32 { }
impl ArrayBufferViewType for i32 { }
impl ArrayBufferViewType for u64 { }
impl ArrayBufferViewType for i64 { }
impl ArrayBufferViewType for f32 { }
impl ArrayBufferViewType for f64 { }

impl<'a> ArrayBufferData<'a> {

    /// Produces an immutable slice as a view into the contents of this buffer.
    /// 
    /// # Example:
    /// 
    /// ```no_run
    /// use neon::js::binary::JsArrayBuffer;
    /// use neon::vm::Context;
    /// use neon::mem::Handle;
    /// # use neon::js::JsUndefined;
    /// # use neon::vm::{JsResult, FunctionContext};
    /// # fn get_x_and_y(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// 
    /// let b: Handle<JsArrayBuffer> = cx.argument(0)?;
    /// let (x, y) = cx.borrow(&b, |data| {
    ///     let slice = data.as_slice::<i32>();
    ///     (slice[0], slice[1])
    /// });
    /// # println!("({}, {})", x, y);
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    pub fn as_slice<T: ArrayBufferViewType>(self) -> &'a [T] {
        let base = unsafe { mem::transmute(self.base) };
        let len = self.size / mem::size_of::<T>();
        unsafe { slice::from_raw_parts(base, len) }
    }

    /// Produces a mutable slice as a view into the contents of this buffer.
    /// 
    /// # Example:
    /// 
    /// ```no_run
    /// use neon::js::binary::JsArrayBuffer;
    /// use neon::vm::Context;
    /// use neon::mem::Handle;
    /// # use neon::js::JsUndefined;
    /// # use neon::vm::{JsResult, FunctionContext};
    /// # fn modify_buffer(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// 
    /// let mut b: Handle<JsArrayBuffer> = cx.argument(0)?;
    /// cx.borrow_mut(&mut b, |data| {
    ///     let slice = data.as_mut_slice::<f64>();
    ///     slice[0] /= 2.0;
    ///     slice[1] *= 2.0;
    /// });
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    pub fn as_mut_slice<T: ArrayBufferViewType>(self) -> &'a mut [T] {
        let base = unsafe { mem::transmute(self.base) };
        let len = self.size / mem::size_of::<T>();
        unsafe { slice::from_raw_parts_mut(base, len) }
    }

    /// Produces the length of the buffer, in bytes.
    pub fn len(self) -> usize {
        self.size
    }
}

impl<'a> Borrow for &'a JsArrayBuffer {
    type Target = ArrayBufferData<'a>;

    fn try_borrow<'b>(self, guard: &'b VmGuard<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        let mut pointer: ArrayBufferData = unsafe { mem::uninitialized() };
        unsafe {
            neon_runtime::arraybuffer::get_data(&mut pointer.base, &mut pointer.size, self.to_raw());
            Ref::new(guard, pointer)
        }
    }
}

impl<'a> Borrow for &'a mut JsArrayBuffer {
    type Target = ArrayBufferData<'a>;

    fn try_borrow<'b>(self, guard: &'b VmGuard<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        (self as &'a JsArrayBuffer).try_borrow(guard)
    }
}

impl<'a> BorrowMut for &'a mut JsArrayBuffer {
    fn try_borrow_mut<'b>(self, guard: &'b VmGuard<'b>) -> Result<RefMut<'b, Self::Target>, LoanError> {
        let mut pointer: ArrayBufferData = unsafe { mem::uninitialized() };
        unsafe {
            neon_runtime::arraybuffer::get_data(&mut pointer.base, &mut pointer.size, self.to_raw());
            RefMut::new(guard, pointer)
        }
    }
}
