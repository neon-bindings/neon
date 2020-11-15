//! Types and traits representing binary JavaScript data.

use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::os::raw::c_void;
use std::slice;
use context::{Context, Lock};
use context::internal::Env;
use borrow::{Borrow, BorrowMut, Ref, RefMut, LoanError};
use borrow::internal::Pointer;
use handle::Managed;
use types::{Value, Object, build};
use types::internal::ValueInternal;
use result::JsResult;
use neon_runtime;
use neon_runtime::raw;

/// The Node [`Buffer`](https://nodejs.org/api/buffer.html) type.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsBuffer(raw::Local);

impl JsBuffer {

    /// Constructs a new `Buffer` object, safely zero-filled.
    pub fn new<'a, C: Context<'a>>(cx: &mut C, size: u32) -> JsResult<'a, JsBuffer> {
        let env = cx.env();
        build(env, |out| { unsafe { neon_runtime::buffer::new(env.to_raw(), out, size) } })
    }

    /// Constructs a new `Buffer` object, safely zero-filled.
    pub unsafe fn uninitialized<'a, C: Context<'a>>(cx: &mut C, size: u32) -> JsResult<'a, JsBuffer> {
        build(cx.env(), |out| { neon_runtime::buffer::uninitialized(out, size) })
    }

}

impl Managed for JsBuffer {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(_env: Env, h: raw::Local) -> Self { JsBuffer(h) }
}

impl ValueInternal for JsBuffer {
    fn name() -> String { "Buffer".to_string() }

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool {
        unsafe { neon_runtime::tag::is_buffer(env.to_raw(), other.to_raw()) }
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
        build(cx.env(), |out| { unsafe { neon_runtime::arraybuffer::new(out, mem::transmute(cx.env()), size) } })
    }

}

impl Managed for JsArrayBuffer {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(_env: Env, h: raw::Local) -> Self { JsArrayBuffer(h) }
}

impl ValueInternal for JsArrayBuffer {
    fn name() -> String { "ArrayBuffer".to_string() }

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool {
        unsafe { neon_runtime::tag::is_arraybuffer(env.to_raw(), other.to_raw()) }
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
        let mut data = MaybeUninit::<BinaryData>::uninit();

        // Initialize pointer
        unsafe {
            let pointer = data.as_mut_ptr();
            (*pointer).size = neon_runtime::buffer::data(guard.env.to_raw(), &mut (*pointer).base, self.to_raw());
        }

        // UB if pointer is not initialized!
        unsafe {
            Ref::new(guard, data.assume_init())
        }
    }
}

impl<'a> Borrow for &'a mut JsBuffer {
    type Target = BinaryData<'a>;

    fn try_borrow<'b>(self, guard: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        (self as &'a JsBuffer).try_borrow(guard)
    }
}

impl<'a> BorrowMut for &'a mut JsBuffer {
    fn try_borrow_mut<'b>(self, guard: &'b Lock<'b>) -> Result<RefMut<'b, Self::Target>, LoanError> {
        let mut data = MaybeUninit::<BinaryData>::uninit();

        // Initialize pointer
        unsafe {
            let pointer = data.as_mut_ptr();
            (*pointer).size = neon_runtime::buffer::data(guard.env.to_raw(), &mut (*pointer).base, self.to_raw());
        }

        // UB if pointer is not initialized!
        unsafe {
            RefMut::new(guard, data.assume_init())
        }
    }
}

impl<'a> Borrow for &'a JsArrayBuffer {
    type Target = BinaryData<'a>;

    fn try_borrow<'b>(self, guard: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        let mut data = MaybeUninit::<BinaryData>::uninit();

        // Initialize pointer
        unsafe {
            let pointer = data.as_mut_ptr();
            (*pointer).size = neon_runtime::arraybuffer::data(guard.env.to_raw(), &mut (*pointer).base, self.to_raw());
        }

        // UB if pointer is not initialized!
        unsafe {
            Ref::new(guard, data.assume_init())
        }
    }
}

impl<'a> Borrow for &'a mut JsArrayBuffer {
    type Target = BinaryData<'a>;

    fn try_borrow<'b>(self, guard: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        (self as &'a JsArrayBuffer).try_borrow(guard)
    }
}

impl<'a> BorrowMut for &'a mut JsArrayBuffer {
    fn try_borrow_mut<'b>(self, guard: &'b Lock<'b>) -> Result<RefMut<'b, Self::Target>, LoanError> {
        let mut data = MaybeUninit::<BinaryData>::uninit();

        // Initialize pointer
        unsafe {
            let pointer = data.as_mut_ptr();
            (*pointer).size = neon_runtime::arraybuffer::data(guard.env.to_raw(), &mut (*pointer).base, self.to_raw());
        }

        // UB if pointer is not initialized!
        unsafe {
            RefMut::new(guard, data.assume_init())
        }
    }
}
