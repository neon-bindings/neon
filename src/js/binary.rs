// pub use internal::js::binary::{JsBuffer, JsArrayBuffer};

use std::mem;
use vm::VmResult;
use js::{Value, Object, build};
use js::internal::ValueInternal;
use mem::{Handle, Managed};
use vm::Lock;
use vm::internal::LockState;
use scope::Scope;
use cslice::CMutSlice;
use neon_runtime;
use neon_runtime::raw;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsBuffer(raw::Local);

impl JsBuffer {
    pub fn new<'a, T: Scope<'a>>(_: &mut T, size: u32) -> VmResult<Handle<'a, JsBuffer>> {
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

impl<'a> Lock for &'a mut JsBuffer {
    type Internals = CMutSlice<'a, u8>;

    unsafe fn expose(self, state: &mut LockState) -> Self::Internals {
        let mut result = mem::uninitialized();
        neon_runtime::buffer::data(&mut result, self.to_raw());
        state.use_buffer(result);
        result
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsArrayBuffer(raw::Local);

impl JsArrayBuffer {
    pub fn new<'a, T: Scope<'a>>(scope: &mut T, size: u32) -> VmResult<Handle<'a, JsArrayBuffer>> {
        build(|out| { unsafe { neon_runtime::arraybuffer::new(out, mem::transmute(scope.isolate()), size) } })
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

impl<'a> Lock for &'a mut JsArrayBuffer {
    type Internals = CMutSlice<'a, u8>;

    unsafe fn expose(self, state: &mut LockState) -> Self::Internals {
        let mut result = mem::uninitialized();
        neon_runtime::arraybuffer::data(&mut result, self.to_raw());
        state.use_buffer(result);
        result
    }
}
