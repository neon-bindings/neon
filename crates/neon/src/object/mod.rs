//! Traits for working with JavaScript objects.
//!
//! This module defines the [`Object`] trait, which is implemented
//! by all object types in the [JavaScript type hierarchy][hierarchy]. This
//! trait provides key operations in the semantics of JavaScript objects,
//! such as getting and setting an object's properties.
//!
//! ## Property Keys
//!
//! Object properties are accessed by a _property key_, which in JavaScript
//! can be a string or [symbol][symbol]. (Neon does not yet have support for
//! symbols.) For convenience, the [`PropertyKey`] trait allows
//! Neon programs to use various Rust string types, as well as numeric types,
//! as keys when accessing object properties, converting the keys to strings
//! as necessary:
//!
//! ```
//! # use neon::prelude::*;
//! fn set_and_check<'a>(
//!     cx: &mut impl Context<'a>,
//!     obj: Handle<'a, JsObject>
//! ) -> JsResult<'a, JsValue> {
//!     let value = cx.string("hello!");
//!     // set property "17" with integer shorthand
//!     obj.set(cx, 17, value)?;
//!     // get property "17" with string shorthand
//!     // returns the same value ("hello!")
//!     obj.get(cx, "17")
//! }
//! ```
//!
//! [hierarchy]: crate::types#the-javascript-type-hierarchy
//! [symbol]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol

use crate::{
    context::Context,
    handle::{Handle, Root},
    result::{NeonResult, Throw},
    sys::{self, raw},
    types::{build, function::CallOptions, utf8::Utf8, JsFunction, JsUndefined, JsValue, Value},
};

#[cfg(feature = "napi-6")]
use crate::{result::JsResult, types::JsArray};

/// A property key in a JavaScript object.
pub trait PropertyKey {
    unsafe fn get_from<'c, C: Context<'c>>(
        self,
        cx: &mut C,
        out: &mut raw::Local,
        obj: raw::Local,
    ) -> bool;

    unsafe fn set_from<'c, C: Context<'c>>(
        self,
        cx: &mut C,
        out: &mut bool,
        obj: raw::Local,
        val: raw::Local,
    ) -> bool;
}

impl PropertyKey for u32 {
    unsafe fn get_from<'c, C: Context<'c>>(
        self,
        cx: &mut C,
        out: &mut raw::Local,
        obj: raw::Local,
    ) -> bool {
        sys::object::get_index(out, cx.env().to_raw(), obj, self)
    }

    unsafe fn set_from<'c, C: Context<'c>>(
        self,
        cx: &mut C,
        out: &mut bool,
        obj: raw::Local,
        val: raw::Local,
    ) -> bool {
        sys::object::set_index(out, cx.env().to_raw(), obj, self, val)
    }
}

impl<'a, K: Value> PropertyKey for Handle<'a, K> {
    unsafe fn get_from<'c, C: Context<'c>>(
        self,
        cx: &mut C,
        out: &mut raw::Local,
        obj: raw::Local,
    ) -> bool {
        let env = cx.env().to_raw();

        sys::object::get(out, env, obj, self.to_local())
    }

    unsafe fn set_from<'c, C: Context<'c>>(
        self,
        cx: &mut C,
        out: &mut bool,
        obj: raw::Local,
        val: raw::Local,
    ) -> bool {
        let env = cx.env().to_raw();

        sys::object::set(out, env, obj, self.to_local(), val)
    }
}

impl<'a> PropertyKey for &'a str {
    unsafe fn get_from<'c, C: Context<'c>>(
        self,
        cx: &mut C,
        out: &mut raw::Local,
        obj: raw::Local,
    ) -> bool {
        let (ptr, len) = Utf8::from(self).into_small_unwrap().lower();
        let env = cx.env().to_raw();

        sys::object::get_string(env, out, obj, ptr, len)
    }

    unsafe fn set_from<'c, C: Context<'c>>(
        self,
        cx: &mut C,
        out: &mut bool,
        obj: raw::Local,
        val: raw::Local,
    ) -> bool {
        let (ptr, len) = Utf8::from(self).into_small_unwrap().lower();
        let env = cx.env().to_raw();

        sys::object::set_string(env, out, obj, ptr, len, val)
    }
}

/// The trait of all object types.
pub trait Object: Value {
    /// Gets a property from a JavaScript object that may be `undefined` and
    /// attempts to downcast the value if it existed.
    fn get_opt<'a, V: Value, C: Context<'a>, K: PropertyKey>(
        &self,
        cx: &mut C,
        key: K,
    ) -> NeonResult<Option<Handle<'a, V>>> {
        let v = self.get_value(cx, key)?;

        if v.is_a::<JsUndefined, _>(cx) {
            return Ok(None);
        }

        v.downcast_or_throw(cx).map(Some)
    }

    /// Gets a property from a JavaScript object as a [`JsValue`].
    ///
    /// If a [`getter`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/get)
    /// is defined on the object, it will be called.
    fn get_value<'a, C: Context<'a>, K: PropertyKey>(
        &self,
        cx: &mut C,
        key: K,
    ) -> NeonResult<Handle<'a, JsValue>> {
        build(cx.env(), move || unsafe {
            let mut out: raw::Local = std::ptr::null_mut();
            key.get_from(cx, &mut out, self.to_local())
                .then_some(out)
                .ok_or(Throw::new())
        })
    }

    /// Gets a property from a JavaScript object and attempts to downcast as a specific type.
    /// Equivalent to calling `obj.get_value(&mut cx)?.downcast_or_throw(&mut cx)`.
    ///
    /// Throws an exception if the value is a different type.
    fn get<'a, V: Value, C: Context<'a>, K: PropertyKey>(
        &self,
        cx: &mut C,
        key: K,
    ) -> NeonResult<Handle<'a, V>> {
        self.get_value(cx, key)?.downcast_or_throw(cx)
    }

    #[cfg(feature = "napi-6")]
    #[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
    fn get_own_property_names<'a, C: Context<'a>>(&self, cx: &mut C) -> JsResult<'a, JsArray> {
        let env = cx.env();

        build(env, move || unsafe {
            let mut out: raw::Local = std::ptr::null_mut();
            sys::object::get_own_property_names(&mut out, env.to_raw(), self.to_local())
                .then_some(out)
                .ok_or(Throw::new())
        })
    }

    #[cfg(feature = "napi-8")]
    fn freeze<'a, C: Context<'a>>(&self, cx: &mut C) -> NeonResult<&Self> {
        let env = cx.env().to_raw();
        let obj = self.to_local();
        unsafe {
            match sys::object::freeze(env, obj) {
                sys::Status::Ok => Ok(self),
                sys::Status::PendingException => Err(Throw::new()),
                _ => cx.throw_type_error("object cannot be frozen"),
            }
        }
    }

    #[cfg(feature = "napi-8")]
    fn seal<'a, C: Context<'a>>(&self, cx: &mut C) -> NeonResult<&Self> {
        let env = cx.env().to_raw();
        let obj = self.to_local();
        unsafe {
            match sys::object::seal(env, obj) {
                sys::Status::Ok => Ok(self),
                sys::Status::PendingException => Err(Throw::new()),
                _ => cx.throw_type_error("object cannot be sealed"),
            }
        }
    }

    fn set<'a, C: Context<'a>, K: PropertyKey, W: Value>(
        &self,
        cx: &mut C,
        key: K,
        val: Handle<W>,
    ) -> NeonResult<bool> {
        let mut result = false;
        unsafe {
            if key.set_from(cx, &mut result, self.to_local(), val.to_local()) {
                Ok(result)
            } else {
                Err(Throw::new())
            }
        }
    }

    fn root<'a, C: Context<'a>>(&self, cx: &mut C) -> Root<Self> {
        Root::new(cx, self)
    }

    fn call_method_with<'a, C, K>(&self, cx: &mut C, method: K) -> NeonResult<CallOptions<'a>>
    where
        C: Context<'a>,
        K: PropertyKey,
    {
        let mut options = self.get::<JsFunction, _, _>(cx, method)?.call_with(cx);
        options.this(JsValue::new_internal(self.to_local()));
        Ok(options)
    }
}
