use std::mem;
use vm::VmResult;
use internal::js::{Value, ValueInternal, Object, build};
use internal::mem::{Handle, Managed};
use internal::vm::{Lock, LockState};
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
