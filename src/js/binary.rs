use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use std::slice;
use vm::{Context, VmResult};
use js::{Value, Object, Borrow, BorrowMut, Ref, RefMut, LoanError, build};
use js::internal::ValueInternal;
use mem::{Handle, Managed};
use vm::VmGuard;
use vm::internal::Pointer;
use neon_runtime;
use neon_runtime::raw;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsBuffer(raw::Local);

impl JsBuffer {
    pub fn new<'a, C: Context<'a>>(_: &mut C, size: u32) -> VmResult<Handle<'a, JsBuffer>> {
        build(|out| { unsafe { neon_runtime::buffer::new(out, size) } })
    }
}

impl Managed for JsBuffer {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsBuffer(h) }
}

impl ValueInternal for JsBuffer {
    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_buffer(other.to_raw()) }
    }
}

impl Value for JsBuffer { }

impl Object for JsBuffer { }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsArrayBuffer(raw::Local);

impl JsArrayBuffer {
    pub fn new<'a, C: Context<'a>>(cx: &mut C, size: u32) -> VmResult<Handle<'a, JsArrayBuffer>> {
        build(|out| { unsafe { neon_runtime::arraybuffer::new(out, mem::transmute(cx.isolate()), size) } })
    }
}

impl Managed for JsArrayBuffer {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsArrayBuffer(h) }
}

impl ValueInternal for JsArrayBuffer {
    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_runtime::tag::is_arraybuffer(other.to_raw()) }
    }
}

impl Value for JsArrayBuffer { }

impl Object for JsArrayBuffer { }

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
    pub fn as_slice<T: ArrayBufferViewType>(self) -> &'a [T] {
        let base = unsafe { mem::transmute(self.base) };
        let len = self.size / mem::size_of::<T>();
        unsafe { slice::from_raw_parts(base, len) }
    }

    pub fn as_mut_slice<T: ArrayBufferViewType>(self) -> &'a mut [T] {
        let base = unsafe { mem::transmute(self.base) };
        let len = self.size / mem::size_of::<T>();
        unsafe { slice::from_raw_parts_mut(base, len) }
    }

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
