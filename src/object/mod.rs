//! Traits for working with JavaScript objects.

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
        fn get<'a, C: Context<'a>, K: PropertyKey>(
            self,
            cx: &mut C,
            key: K,
        ) -> NeonResult<Handle<'a, JsValue>> {
            build(cx.env(), |out| unsafe { key.get_from(out, self.to_raw()) })
        }

        fn get_own_property_names<'a, C: Context<'a>>(self, cx: &mut C) -> JsResult<'a, JsArray> {
            let env = cx.env();
            build(env, |out| unsafe {
                neon_runtime::object::get_own_property_names(out, env.to_raw(), self.to_raw())
            })
        }

        fn set<'a, C: Context<'a>, K: PropertyKey, W: Value>(
            self,
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
        fn get<'a, C: Context<'a>, K: PropertyKey>(
            self,
            cx: &mut C,
            key: K,
        ) -> NeonResult<Handle<'a, JsValue>> {
            build(cx.env(), |out| unsafe {
                key.get_from(cx, out, self.to_raw())
            })
        }

        #[cfg(feature = "napi-6")]
        fn get_own_property_names<'a, C: Context<'a>>(self, cx: &mut C) -> JsResult<'a, JsArray> {
            let env = cx.env();

            build(cx.env(), |out| unsafe {
                neon_runtime::object::get_own_property_names(out, env.to_raw(), self.to_raw())
            })
        }

        fn set<'a, C: Context<'a>, K: PropertyKey, W: Value>(
            self,
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
