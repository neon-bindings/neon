use std::mem;
use std::ffi::CString;

use neon_sys;
use neon_sys::raw;

use internal::vm::{Throw, Result};
use internal::value::{SomeObject, Any, AnyInternal, Object, build};
use internal::mem::Handle;
use scope::Scope;

pub fn throw<'a, T: Any, U>(v: Handle<'a, T>) -> Result<U> {
    unsafe {
        neon_sys::error::throw(v.to_raw());
    }
    Err(Throw)
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TypeError(raw::Local);

impl AnyInternal for TypeError {
    fn to_raw(self) -> raw::Local { self.0 }

    fn from_raw(h: raw::Local) -> Self { TypeError(h) }

    fn is_typeof<Other: Any>(other: Other) -> bool {
        unsafe { neon_sys::tag::is_type_error(other.to_raw()) }
    }
}

impl Any for TypeError { }

impl Object for TypeError { }

fn message(msg: &str) -> CString {
    CString::new(msg).ok().unwrap_or_else(|| { CString::new("").ok().unwrap() })
}

impl TypeError {
    // FIXME: use an overload trait to allow either &str or value::String
    pub fn new<'a, T: Scope<'a>>(_: &mut T, msg: &str) -> Result<Handle<'a, SomeObject>> {
        let msg = &message(msg);
        build(|out| { unsafe { neon_sys::error::new_type_error(out, mem::transmute(msg.as_ptr())) } })
    }

    pub fn throw<T>(msg: &str) -> Result<T> {
        let msg = &message(msg);
        unsafe {
            neon_sys::error::throw_type_error(mem::transmute(msg.as_ptr()));
        }
        Err(Throw)
    }
}
