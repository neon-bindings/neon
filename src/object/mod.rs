//! Traits for working with JavaScript objects.
//!
//! This module defines the [`Object`](Object) trait, which is implemented
//! by all object types in the [JavaScript type hierarchy][hierarchy]. This
//! trait provides key operations in the semantics of JavaScript objects,
//! such as getting and setting an object's properties.
//!
//! ## Property Keys
//!
//! Object properties are accessed by a _property key_, which in JavaScript
//! can be a string or [symbol][symbol]. (Neon does not yet have support for
//! symbols.) For convenience, the [`PropertyKey`](PropertyKey) trait allows
//! Neon programs to use various Rust string types, as well as numeric types,
//! as keys when accessing object properties, converting the keys to strings
//! as necessary:
//!
//! ```
//! # #[cfg(feature = "napi-1")] {
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
//! # }
//! ```
//!
//! [hierarchy]: crate::types#the-javascript-type-hierarchy
//! [symbol]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol

#[cfg(feature = "legacy-runtime")]
pub(crate) mod class;

#[cfg(feature = "legacy-runtime")]
pub use self::class::{Class, ClassDescriptor};
pub use self::traits::*;

#[cfg(feature = "legacy-runtime")]
mod traits {
    use crate::context::Context;
    use crate::handle::{Handle, Managed};
    use crate::result::{JsResult, NeonResult, Throw};
    use crate::types::utf8::Utf8;
    use crate::types::{build, JsArray, JsValue, Value};
    use neon_runtime::raw;

    /// A property key in a JavaScript object.
    pub trait PropertyKey {
        unsafe fn get_from(self, out: &mut raw::Local, obj: raw::Local) -> bool;
        unsafe fn set_from(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool;
    }

    impl PropertyKey for u32 {
        unsafe fn get_from(self, out: &mut raw::Local, obj: raw::Local) -> bool {
            neon_runtime::object::get_index(out, obj, self)
        }

        unsafe fn set_from(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
            neon_runtime::object::set_index(out, obj, self, val)
        }
    }

    impl<'a, K: Value> PropertyKey for Handle<'a, K> {
        unsafe fn get_from(self, out: &mut raw::Local, obj: raw::Local) -> bool {
            neon_runtime::object::get(out, obj, self.to_raw())
        }

        unsafe fn set_from(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
            neon_runtime::object::set(out, obj, self.to_raw(), val)
        }
    }

    impl<'a> PropertyKey for &'a str {
        unsafe fn get_from(self, out: &mut raw::Local, obj: raw::Local) -> bool {
            let (ptr, len) = Utf8::from(self).into_small_unwrap().lower();
            neon_runtime::object::get_string(out, obj, ptr, len)
        }

        unsafe fn set_from(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
            let (ptr, len) = Utf8::from(self).into_small_unwrap().lower();
            neon_runtime::object::set_string(out, obj, ptr, len, val)
        }
    }

    /// The trait of all object types.
    pub trait Object: Value {
        fn get<'a, V: Value, C: Context<'a>, K: PropertyKey>(
            &self,
            cx: &mut C,
            key: K,
        ) -> NeonResult<Handle<'a, V>> {
            let v: Handle<JsValue> =
                build(cx.env(), |out| unsafe { key.get_from(out, self.to_raw()) })?;
            v.downcast_or_throw(cx)
        }

        fn get_own_property_names<'a, C: Context<'a>>(&self, cx: &mut C) -> JsResult<'a, JsArray> {
            let env = cx.env();
            build(env, |out| unsafe {
                neon_runtime::object::get_own_property_names(out, env.to_raw(), self.to_raw())
            })
        }

        fn set<'a, C: Context<'a>, K: PropertyKey, W: Value>(
            &self,
            _: &mut C,
            key: K,
            val: Handle<W>,
        ) -> NeonResult<bool> {
            let mut result = false;
            if unsafe { key.set_from(&mut result, self.to_raw(), val.to_raw()) } {
                Ok(result)
            } else {
                Err(Throw)
            }
        }
    }

    /// The trait of types that can be a function's `this` binding.
    pub unsafe trait This: Managed {
        #[allow(clippy::wrong_self_convention)]
        fn as_this(h: raw::Local) -> Self;
    }
}

#[cfg(feature = "napi-1")]
mod traits {
    use crate::context::internal::Env;
    use crate::context::Context;
    use crate::handle::{Handle, Managed, Root};
    use crate::result::{NeonResult, Throw};
    use crate::types::utf8::Utf8;
    use crate::types::{build, JsValue, Value};
    use neon_runtime::raw;

    #[cfg(feature = "napi-6")]
    use crate::result::JsResult;
    #[cfg(feature = "napi-6")]
    use crate::types::JsArray;

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
            neon_runtime::object::get_index(out, cx.env().to_raw(), obj, self)
        }

        unsafe fn set_from<'c, C: Context<'c>>(
            self,
            cx: &mut C,
            out: &mut bool,
            obj: raw::Local,
            val: raw::Local,
        ) -> bool {
            neon_runtime::object::set_index(out, cx.env().to_raw(), obj, self, val)
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

            neon_runtime::object::get(out, env, obj, self.to_raw())
        }

        unsafe fn set_from<'c, C: Context<'c>>(
            self,
            cx: &mut C,
            out: &mut bool,
            obj: raw::Local,
            val: raw::Local,
        ) -> bool {
            let env = cx.env().to_raw();

            neon_runtime::object::set(out, env, obj, self.to_raw(), val)
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

            neon_runtime::object::get_string(env, out, obj, ptr, len)
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

            neon_runtime::object::set_string(env, out, obj, ptr, len, val)
        }
    }

    /// The trait of all object types.
    pub trait Object: Value {
        fn get<'a, V: Value, C: Context<'a>, K: PropertyKey>(
            &self,
            cx: &mut C,
            key: K,
        ) -> NeonResult<Handle<'a, V>> {
            let v: Handle<JsValue> = build(cx.env(), |out| unsafe {
                key.get_from(cx, out, self.to_raw())
            })?;
            v.downcast_or_throw(cx)
        }

        #[cfg(feature = "napi-6")]
        #[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
        fn get_own_property_names<'a, C: Context<'a>>(&self, cx: &mut C) -> JsResult<'a, JsArray> {
            let env = cx.env();

            build(cx.env(), |out| unsafe {
                neon_runtime::object::get_own_property_names(out, env.to_raw(), self.to_raw())
            })
        }

        fn set<'a, C: Context<'a>, K: PropertyKey, W: Value>(
            &self,
            cx: &mut C,
            key: K,
            val: Handle<W>,
        ) -> NeonResult<bool> {
            let mut result = false;
            if unsafe { key.set_from(cx, &mut result, self.to_raw(), val.to_raw()) } {
                Ok(result)
            } else {
                Err(Throw)
            }
        }

        fn root<'a, C: Context<'a>>(&self, cx: &mut C) -> Root<Self> {
            Root::new(cx, self)
        }
    }

    /// The trait of types that can be a function's `this` binding.
    pub unsafe trait This: Managed {
        #[allow(clippy::wrong_self_convention)]
        fn as_this(env: Env, h: raw::Local) -> Self;
    }
}
