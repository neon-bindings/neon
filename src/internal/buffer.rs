use std::ops::{Index, IndexMut};
use std::str;
use std::str::Utf8Error;

use vm::Throw;
use internal::error::TypeError;
use internal::value::{SomeObject, Any, AnyInternal, Object, build};
use internal::mem::Handle;
use internal::vm::{Lock, LockState};
use scope::Scope;
use neon_sys::raw;
use neon_sys::{NeonSys_NewBuffer, NeonSys_Buffer_Data, NeonSys_IsBuffer};
use neon_sys::buf::Buf;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Buffer(raw::Local);

impl Buffer {
    pub fn new<'a, T: Scope<'a>>(_: &mut T, size: u32) -> Result<Handle<'a, SomeObject>, Throw> {
        build(|out| { unsafe { NeonSys_NewBuffer(out, size) } })
    }
}

impl AnyInternal for Buffer {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { Buffer(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { NeonSys_IsBuffer(other.to_raw()) }
    }
}

impl Any for Buffer { }

impl Object for Buffer { }

impl<'a> Lock for Handle<'a, Buffer> {
    type Internals = Buf<'a>;

    unsafe fn expose(self, state: &mut LockState) -> Self::Internals {
        let mut result = Buf::uninitialized();
        NeonSys_Buffer_Data(&mut result, self.to_raw());
        state.use_buffer(&result);
        result
    }
}
