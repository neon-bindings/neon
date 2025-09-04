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
//! fn set_and_check<'cx>(
//!     cx: &mut Cx<'cx>,
//!     obj: Handle<'cx, JsObject>
//! ) -> JsResult<'cx, JsValue> {
//!     // set property "17" with integer shorthand
//!     obj.prop(cx, 17).set("hello")?;
//!     // get property "17" with string shorthand
//!     // returns the same value ("hello!")
//!     obj.prop(cx, "17").get()
//! }
//! ```
//!
//! [hierarchy]: crate::types#the-javascript-type-hierarchy
//! [symbol]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol

use smallvec::smallvec;

use crate::{
    context::{internal::ContextInternal, Context, Cx},
    handle::{Handle, Root},
    result::{NeonResult, Throw},
    sys::{self, raw},
    types::{
        build,
        extract::{TryFromJs, TryIntoJs},
        function::{BindOptions, CallOptions},
        private::ValueInternal,
        utf8::Utf8,
        JsFunction, JsUndefined, JsValue, Value,
    },
};

#[cfg(feature = "napi-6")]
use crate::{result::JsResult, types::JsArray};

#[cfg(feature = "napi-6")]
pub use self::class::{Class, ClassMetadata};

#[cfg(feature = "napi-6")]
#[doc(hidden)]
pub use self::class::RootClassMetadata;

#[doc(hidden)]
pub use self::wrap::{unwrap, wrap};

#[cfg(feature = "napi-6")]
pub(crate) mod class;
pub(crate) mod wrap;

/// A property key in a JavaScript object.
pub trait PropertyKey: Copy {
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

/// A builder for accessing an object property.
///
/// The builder methods make it convenient to get and set properties
/// as well as to bind and call methods.
/// ```
/// # use neon::prelude::*;
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsString> {
/// # let obj: Handle<JsObject> = cx.argument(0)?;
/// let x: f64 = obj
///     .prop(&mut cx, "x")
///     .get()?;
///
/// obj.prop(&mut cx, "y")
///     .set(x)?;
///
/// let s: String = obj.method(&mut cx, "toString")?.call()?;
/// # Ok(cx.string(s))
/// # }
/// ```
pub struct PropOptions<'a, 'cx, O, K>
where
    'cx: 'a,
    O: Object,
    K: PropertyKey,
{
    pub(crate) cx: &'a mut Cx<'cx>,
    pub(crate) this: Handle<'cx, O>,
    pub(crate) key: K,
}

impl<'a, 'cx, O, K> PropOptions<'a, 'cx, O, K>
where
    'cx: 'a,
    O: Object,
    K: PropertyKey,
{
    /// Returns the original object from which the property was accessed.
    pub fn this(&self) -> Handle<'cx, O> {
        self.this
    }

    /// Updates the property key.
    ///
    /// This method is useful for chaining multiple property assignments:
    ///
    /// ```
    /// # use neon::prelude::*;
    /// # fn foo(mut cx: FunctionContext) -> JsResult<JsObject> {
    /// let obj = cx.empty_object()
    ///     .prop(&mut cx, "x")
    ///     .set(1)?
    ///     .prop("y")
    ///     .set(2)?
    ///     .prop("color")
    ///     .set("blue")?
    ///     .this();
    /// # Ok(obj)
    /// # }
    /// ```
    pub fn prop(&mut self, key: K) -> &mut Self {
        self.key = key;
        self
    }

    /// Gets the property from the object and attempts to convert it to a Rust value.
    ///
    /// May throw an exception either during accessing the property or converting the
    /// result type.
    pub fn get<R: TryFromJs<'cx>>(&mut self) -> NeonResult<R> {
        let v = self.this.get_value(self.cx, self.key)?;
        R::from_js(self.cx, v)
    }

    /// Sets the property on the object to a value converted from Rust.
    ///
    /// May throw an exception either during converting the value or setting the property.
    pub fn set<V: TryIntoJs<'cx>>(&mut self, v: V) -> NeonResult<&mut Self> {
        let v = v.try_into_js(self.cx)?;
        self.this.set(self.cx, self.key, v)?;
        Ok(self)
    }

    /// Sets the property on the object to a value computed from a closure.
    ///
    /// May throw an exception either during converting the value or setting the property.
    pub fn set_with<R, F>(&mut self, f: F) -> NeonResult<&mut Self>
    where
        R: TryIntoJs<'cx>,
        F: FnOnce(&mut Cx<'cx>) -> R,
    {
        let v = f(self.cx).try_into_js(self.cx)?;
        self.this.set(self.cx, self.key, v)?;
        Ok(self)
    }

    /// Gets the property from the object as a method and binds `this` to the object.
    ///
    /// May throw an exception when accessing the property.
    ///
    /// Defers checking that the method is callable until call time.
    pub fn bind(&'a mut self) -> NeonResult<BindOptions<'a, 'cx>> {
        let callee: Handle<JsValue> = self.this.get(self.cx, self.key)?;
        let this = Some(self.this.upcast());
        Ok(BindOptions {
            cx: self.cx,
            callee,
            this,
            args: smallvec![],
        })
    }
}

/// The trait of all object types.
pub trait Object: Value {
    /// Create a [`PropOptions`] for accessing a property.
    ///
    /// # Safety
    ///
    /// Because `cx` is a mutable reference, Neon guarantees it
    /// is the context with the shortest possible lifetime, so
    /// replacing the lifetime `'self` with `'cx` cannot extend
    /// the lifetime of the property beyond the lifetime of the
    /// object.
    fn prop<'a, 'cx: 'a, K: PropertyKey>(
        &self,
        cx: &'a mut Cx<'cx>,
        key: K,
    ) -> PropOptions<'a, 'cx, Self, K> {
        let this: Handle<'_, Self> =
            Handle::new_internal(unsafe { ValueInternal::from_local(cx.env(), self.to_local()) });
        PropOptions { cx, this, key }
    }

    /// Gets a property from the object as a method and binds `this` to the object.
    ///
    /// May throw an exception either from accessing the property.
    ///
    /// Defers checking that the method is callable until call time.
    fn method<'a, 'cx: 'a, K: PropertyKey>(
        &self,
        cx: &'a mut Cx<'cx>,
        key: K,
    ) -> NeonResult<BindOptions<'a, 'cx>> {
        let callee: Handle<JsValue> = self.prop(cx, key).get()?;
        let this = Some(self.as_value(cx));
        Ok(BindOptions {
            cx,
            callee,
            this,
            args: smallvec![],
        })
    }

    #[deprecated(since = "TBD", note = "use `Object::prop()` instead")]
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

    #[deprecated(since = "TBD", note = "use `Object::prop()` instead")]
    fn get_value<'a, C: Context<'a>, K: PropertyKey>(
        &self,
        cx: &mut C,
        key: K,
    ) -> NeonResult<Handle<'a, JsValue>> {
        build(cx.env(), |out| unsafe {
            key.get_from(cx, out, self.to_local())
        })
    }

    #[deprecated(since = "TBD", note = "use `Object::prop()` instead")]
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

        build(cx.env(), |out| unsafe {
            sys::object::get_own_property_names(out, env.to_raw(), self.to_local())
        })
    }

    #[cfg(feature = "napi-8")]
    fn freeze<'a, C: Context<'a>>(&self, cx: &mut C) -> NeonResult<&Self> {
        let env = cx.env().to_raw();
        let obj = self.to_local();
        unsafe {
            match sys::object::freeze(env, obj) {
                Ok(()) => Ok(self),
                Err(sys::Status::PendingException) => Err(Throw::new()),
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
                Ok(()) => Ok(self),
                Err(sys::Status::PendingException) => Err(Throw::new()),
                _ => cx.throw_type_error("object cannot be sealed"),
            }
        }
    }

    #[deprecated(since = "TBD", note = "use `Object::prop()` instead")]
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

    #[deprecated(since = "TBD", note = "use `Object::method()` instead")]
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
