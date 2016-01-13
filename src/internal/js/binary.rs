use vm::VmResult;
use internal::js::{Value, ValueInternal, Object, build};
use internal::mem::{Handle, Managed};
use internal::vm::{Lock, LockState};
use scope::Scope;
use neon_sys;
use neon_sys::raw;
use neon_sys::buf::Buf;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsBuffer(raw::Local);

impl JsBuffer {
    pub fn new<'a, T: Scope<'a>>(_: &mut T, size: u32) -> VmResult<Handle<'a, JsBuffer>> {
        build(|out| { unsafe { neon_sys::buffer::new(out, size) } })
    }
}

impl Managed for JsBuffer {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { JsBuffer(h) }
}

impl ValueInternal for JsBuffer {
    fn is_typeof<Other: Value>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_buffer(other.to_raw()) }
    }
}

impl Value for JsBuffer { }

impl Object for JsBuffer { }

impl<'a> Lock for Handle<'a, JsBuffer> {
    type Internals = Buf<'a>;

    unsafe fn expose(self, state: &mut LockState) -> Self::Internals {
        let mut result = Buf::uninitialized();
        neon_sys::buffer::data(&mut result, self.to_raw());
        state.use_buffer(&result);
        result
    }
}
