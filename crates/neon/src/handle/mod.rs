//! References to garbage-collected JavaScript values.
//!
//! A _handle_ is a safe reference to a JavaScript value that is owned and managed
//! by the JavaScript engine's memory management system (the garbage collector).
//!
//! Neon APIs that accept and return JavaScript values never use raw pointer types
//! ([`*T`](pointer)) or reference types ([`&T`](reference)). Instead they use the
//! special Neon type [`Handle`](Handle), which encapsulates a JavaScript
//! [`Value`](crate::types::Value) and ensures that Rust only maintains access to
//! the value while it is guaranteed to be valid.
//!
//! ## Working with Handles
//!
//! The `Handle<T>` type automatically dereferences to `T` (via the standard
//! [`Deref`](std::ops::Deref) trait), so you can call `T`'s methods on a value of
//! type `Handle<T>`. For example, we can call
//! [`JsNumber::value()`](crate::types::JsNumber::value) on a `Handle<JsNumber>`:
//!
//! ```
//! # use neon::prelude::*;
//! # fn run(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//! let n: Handle<JsNumber> = cx.argument(0)?;
//! let v = n.value(&mut cx); // JsNumber::value()
//! # Ok(cx.undefined())
//! # }
//! ```
//!
//! ## Example
//!
//! This Neon function takes an object as its argument, extracts two properties,
//! `width` and `height`, and multiplies them together as numbers. Each JavaScript
//! value in the calculation is stored locally in a `Handle`.
//!
//! ```
//! # use neon::prelude::*;
//! fn area(mut cx: FunctionContext) -> JsResult<JsNumber> {
//!     let rect: Handle<JsObject> = cx.argument(0)?;
//!
//!     let width: Handle<JsNumber> = rect.get(&mut cx, "width")?;
//!     let w: f64 = width.value(&mut cx);
//!
//!     let height: Handle<JsNumber> = rect.get(&mut cx, "height")?;
//!     let h: f64 = height.value(&mut cx);
//!
//!     Ok(cx.number(w * h))
//! }
//! ```

pub(crate) mod internal;

pub(crate) mod root;

use std::{
    error::Error,
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
};

pub use self::root::Root;

use crate::{
    context::{internal::Env, Context},
    handle::internal::{SuperType, TransparentNoCopyWrapper},
    result::{JsResult, JsResultExt},
    sys::{self, raw},
    types::Value,
};

/// The trait of data owned by the JavaScript engine and that can only be accessed via handles.
pub trait Managed: TransparentNoCopyWrapper {
    fn to_raw(&self) -> raw::Local;

    fn from_raw(env: Env, h: raw::Local) -> Self;
}

/// A handle to a JavaScript value that is owned by the JavaScript engine.
#[derive(Debug)]
#[repr(transparent)]
pub struct Handle<'a, T: Managed + 'a> {
    // Contains the actual `Copy` JavaScript value data. It will be wrapped in
    // in a `!Copy` type when dereferencing. Only `T` should be visible to the user.
    value: <T as TransparentNoCopyWrapper>::Inner,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: Managed> Clone for Handle<'a, T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T: Managed> Copy for Handle<'a, T> {}

impl<'a, T: Managed + 'a> Handle<'a, T> {
    pub(crate) fn new_internal(value: T) -> Handle<'a, T> {
        Handle {
            value: value.into_inner(),
            phantom: PhantomData,
        }
    }
}

/// An error representing a failed downcast.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct DowncastError<F: Value, T: Value> {
    phantom_from: PhantomData<F>,
    phantom_to: PhantomData<T>,
}

impl<F: Value, T: Value> Debug for DowncastError<F, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "DowncastError")
    }
}

impl<F: Value, T: Value> DowncastError<F, T> {
    fn new() -> Self {
        DowncastError {
            phantom_from: PhantomData,
            phantom_to: PhantomData,
        }
    }
}

impl<F: Value, T: Value> Display for DowncastError<F, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "failed to downcast {} to {}", F::name(), T::name())
    }
}

impl<F: Value, T: Value> Error for DowncastError<F, T> {}

/// The result of a call to [`Handle::downcast()`](Handle::downcast).
pub type DowncastResult<'a, F, T> = Result<Handle<'a, T>, DowncastError<F, T>>;

impl<'a, F: Value, T: Value> JsResultExt<'a, T> for DowncastResult<'a, F, T> {
    fn or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> JsResult<'a, T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => cx.throw_type_error(&e.to_string()),
        }
    }
}

impl<'a, T: Value> Handle<'a, T> {
    /// Safely upcast a handle to a supertype.
    ///
    /// This method does not require an execution context because it only copies a handle.
    pub fn upcast<U: Value + SuperType<T>>(&self) -> Handle<'a, U> {
        Handle::new_internal(SuperType::upcast_internal(self.deref()))
    }

    /// Tests whether this value is an instance of the given type.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// # use neon::prelude::*;
    /// # fn my_neon_function(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// let v: Handle<JsValue> = cx.number(17).upcast();
    /// v.is_a::<JsString, _>(&mut cx); // false
    /// v.is_a::<JsNumber, _>(&mut cx); // true
    /// v.is_a::<JsValue, _>(&mut cx);  // true
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    pub fn is_a<'b, U: Value, C: Context<'b>>(&self, cx: &mut C) -> bool {
        U::is_typeof(cx.env(), self.deref())
    }

    /// Attempts to downcast a handle to another type, which may fail. A failure
    /// to downcast **does not** throw a JavaScript exception, so it's OK to
    /// continue interacting with the JS engine if this method produces an `Err`
    /// result.
    pub fn downcast<'b, U: Value, C: Context<'b>>(&self, cx: &mut C) -> DowncastResult<'a, T, U> {
        match U::downcast(cx.env(), self.deref()) {
            Some(v) => Ok(Handle::new_internal(v)),
            None => Err(DowncastError::new()),
        }
    }

    /// Attempts to downcast a handle to another type, raising a JavaScript `TypeError`
    /// exception on failure. This method is a convenient shorthand, equivalent to
    /// `self.downcast::<U>().or_throw::<C>(cx)`.
    pub fn downcast_or_throw<'b, U: Value, C: Context<'b>>(&self, cx: &mut C) -> JsResult<'a, U> {
        self.downcast(cx).or_throw(cx)
    }

    pub fn strict_equals<'b, U: Value, C: Context<'b>>(
        &self,
        cx: &mut C,
        other: Handle<'b, U>,
    ) -> bool {
        unsafe { sys::mem::strict_equals(cx.env().to_raw(), self.to_raw(), other.to_raw()) }
    }
}

impl<'a, T: Managed> Deref for Handle<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { mem::transmute(&self.value) }
    }
}

impl<'a, T: Managed> DerefMut for Handle<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { mem::transmute(&mut self.value) }
    }
}
