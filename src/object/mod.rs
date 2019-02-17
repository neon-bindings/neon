//! Traits for working with JavaScript objects.

pub(crate) mod class;

use neon_runtime;
use neon_runtime::raw;
use types::{JsValue, JsArray, Managed, Value};
use types::utf8::Utf8;
use context::Context;
use result::{NeonResult, Throw};

pub use self::class::{Class, ClassDescriptor};

/// A property key in a JavaScript object.
pub trait PropertyKey {
    unsafe fn get_from(self, out: &raw::Persistent, isolate: *mut raw::Isolate, obj: &raw::Persistent) -> bool;
    unsafe fn set_from(self, out: &mut bool, isolate: *mut raw::Isolate, obj: &raw::Persistent, val: &raw::Persistent) -> bool;
}

impl PropertyKey for u32 {
    unsafe fn get_from(self, out: &raw::Persistent, isolate: *mut raw::Isolate, obj: &raw::Persistent) -> bool {
        neon_runtime::object::get_index(out, isolate, obj, self)
    }

    unsafe fn set_from(self, out: &mut bool, isolate: *mut raw::Isolate, obj: &raw::Persistent, val: &raw::Persistent) -> bool {
        neon_runtime::object::set_index(out, isolate, obj, self, val)
    }
}

impl<'a> PropertyKey for &'a str {
    unsafe fn get_from(self, out: &raw::Persistent, isolate: *mut raw::Isolate, obj: &raw::Persistent) -> bool {
        let (ptr, len) = Utf8::from(self).into_small_unwrap().lower();
        // FIXME: rename the `_thin` back to normal
        neon_runtime::object::get_string_thin(out, isolate, obj, ptr, len)
    }

    unsafe fn set_from(self, out: &mut bool, isolate: *mut raw::Isolate, obj: &raw::Persistent, val: &raw::Persistent) -> bool {
        let (ptr, len) = Utf8::from(self).into_small_unwrap().lower();
        // FIXME: rename the `_thin` back to normal
        neon_runtime::object::set_string_thin(out, isolate, obj, ptr, len, val)
    }
}

impl<'a, T: Value> PropertyKey for &'a T {
    unsafe fn get_from(self, out: &raw::Persistent, isolate: *mut raw::Isolate, obj: &raw::Persistent) -> bool {
        // FIXME: rename the `_thin` back to normal
        neon_runtime::object::get_thin(out, isolate, obj, self.to_raw())
    }

    unsafe fn set_from(self, out: &mut bool, isolate: *mut raw::Isolate, obj: &raw::Persistent, val: &raw::Persistent) -> bool {
        // FIXME: rename the `_thin` back to normal
        neon_runtime::object::set_thin(out, isolate, obj, self.to_raw(), val)
    }
}

pub trait Object: Value {
    fn get<'a, C: Context<'a>, K: PropertyKey>(&self, cx: &mut C, key: K) -> NeonResult<&'a JsValue> {
        cx.new(|out, isolate| { unsafe { key.get_from(out, isolate, self.to_raw()) } })
    }

    fn get_own_property_names<'a, C: Context<'a>>(&self, cx: &mut C) -> NeonResult<&'a JsArray> {
        cx.new(|out, isolate| unsafe {
            neon_runtime::object::get_own_property_names(out, isolate, self.to_raw())
        })
    }

    fn set<'a, C: Context<'a>, K: PropertyKey, W: Value>(&self, cx: &mut C, key: K, val: &W) -> NeonResult<bool> {
        let mut result = false;
        let isolate = { cx.isolate().to_raw() };
        if unsafe { key.set_from(&mut result, isolate, self.to_raw(), val.to_raw()) } {
            Ok(result)
        } else {
            Err(Throw)
        }
    }
}

/// The trait of types that can be a function's `this` binding.
pub unsafe trait This: Managed {
    fn as_this(h: &raw::Persistent) -> &Self {
        Self::from_raw(h)
    }
}
