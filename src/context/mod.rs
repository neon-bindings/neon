//! Provides runtime access to the JavaScript engine.
//!
//! An _execution context_ represents the current state of a thread of execution in the
//! JavaScript engine. Internally, it tracks things like the set of pending function calls,
//! whether the engine is currently throwing an exception or not, and whether the engine is
//! in the process of shutting down. The context uses this internal state to manage what
//! operations are safely available and when.
//!
//! The [`Context`](Context) trait provides an abstract interface to the JavaScript
//! execution context. All interaction with the JavaScript engine in Neon code is mediated
//! through instances of this trait.
//!
//! One particularly useful context type is [`CallContext`](CallContext), which is passed
//! to all Neon functions as their initial execution context (or [`FunctionContext`](FunctionContext),
//! a convenient shorthand for `CallContext<JsObject>`):
//!
//! ```
//! # use neon::prelude::*;
//! fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
//!     Ok(cx.string("hello Neon"))
//! }
//! ```
//!
//! Another important context type is [`ModuleContext`](ModuleContext), which is provided
//! to a Neon module's [`main`](crate::main) function to enable sharing Neon functions back
//! with JavaScript:
//!
//! ```
//! # #[cfg(feature = "neon-macros")] {
//! # use neon::prelude::*;
//! # fn hello(_: FunctionContext) -> JsResult<JsValue> { todo!() }
//! #[neon::main]
//! fn main(mut cx: ModuleContext) -> NeonResult<()> {
//!     cx.export_function("hello", hello)?;
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ## Memory Management
//!
//! Because contexts represent the engine at a point in time, they are associated with a
//! [_lifetime_][lifetime], which limits how long Rust code is allowed to access them. This
//! is also used to determine the lifetime of [`Handle`](crate::handle::Handle)s, which
//! provide safe references to JavaScript memory managed by the engine's garbage collector.
//!
//! For example, we can
//! write a simple string scanner that counts whitespace in a JavaScript string and
//! returns a [`JsNumber`](crate::types::JsNumber):
//!
//! ```
//! # #[cfg(feature = "napi-1")] {
//! # use neon::prelude::*;
//! fn count_whitespace(mut cx: FunctionContext) -> JsResult<JsNumber> {
//!     let s: Handle<JsString> = cx.argument(0)?;
//!     let contents = s.value(&mut cx);
//!     let count = contents
//!         .chars()                       // iterate over the characters
//!         .filter(|c| c.is_whitespace()) // select the whitespace chars
//!         .count();                      // count the resulting chars
//!     Ok(cx.number(count as f64))
//! }
//! # }
//! ```
//!
//! In this example, `s` is assigned a handle to a string, which ensures that the string
//! is _kept alive_ (i.e., prevented from having its storage reclaimed by the JavaScript
//! engine's garbage collector) for the duration of the `count_whitespace` function. This
//! is how Neon takes advantage of Rust's type system to allow your Rust code to safely
//! interact with JavaScript values.
//!
//! ### Temporary Scopes
//!
//! Sometimes it can be useful to limit the scope of a handle's lifetime, to allow the
//! engine to reclaim memory sooner. This can be important when, for example, an expensive inner loop generates
//! temporary JavaScript values that are only needed inside the loop. In these cases,
//! the [`execute_scoped`](Context::execute_scoped) and [`compute_scoped`](Context::compute_scoped)
//! methods allow you to create temporary contexts in order to allocate temporary
//! handles.
//!
//! For example, to extract the elements of a JavaScript [iterator][iterator] from Rust,
//! a Neon function has to work with several temporary handles on each pass through
//! the loop:
//!
//! ```
//! # #[cfg(feature = "napi-1")] {
//! # use neon::prelude::*;
//! # fn iterate(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//!     let iterator = cx.argument::<JsObject>(0)?;         // iterator object
//!     let next = iterator.get(&mut cx, "next")?           // iterator's `next` method
//!         .downcast_or_throw::<JsFunction, _>(&mut cx)?;
//!     let mut numbers = vec![];                           // results vector
//!     let mut done = false;                               // loop controller
//!
//!     while !done {
//!         done = cx.execute_scoped(|mut cx| {                   // temporary scope
//!             let args: Vec<Handle<JsValue>> = vec![];
//!             let obj = next.call(&mut cx, iterator, args)?     // temporary object
//!                 .downcast_or_throw::<JsObject, _>(&mut cx)?;
//!             let number = obj.get(&mut cx, "value")?           // temporary number
//!                 .downcast_or_throw::<JsNumber, _>(&mut cx)?
//!                 .value(&mut cx);
//!             numbers.push(number);
//!             Ok(obj.get(&mut cx, "done")?                      // temporary boolean
//!                 .downcast_or_throw::<JsBoolean, _>(&mut cx)?
//!                 .value(&mut cx))
//!         })?;
//!     }
//! #   Ok(cx.undefined())
//! # }
//! # }
//! ```
//!
//! The temporary scope ensures that the temporary values are only kept alive
//! during a single pass through the loop, since the temporary context is
//! discarded (and all of its handles released) on the inside of the loop.
//!
//! ## Throwing Exceptions
//!
//! When a Neon API causes a JavaScript exception to be thrown, it returns an
//! [`Err`](std::result::Result::Err) result, indicating that the thread associated
//! with the context is now throwing. This allows Rust code to perform any
//! cleanup before returning, but with an important restriction:
//!
//! > **While a JavaScript thread is throwing, its context cannot be used.**
//!
//! Unless otherwise documented, any Neon API that uses a context (as `self` or as
//! a parameter) immediately panics if called while the context's thread is throwing.
//!
//! Typically, Neon code can manage JavaScript exceptions correctly and conveniently
//! by using Rust's [question mark (`?`)][question-mark] operator. This ensures that
//! Rust code "short-circuits" when an exception is thrown and returns back to
//! JavaScript without calling any throwing APIs.
//!
//! Alternatively, to invoke a Neon API and catch any JavaScript exceptions, use the
//! [`Context::try_catch`](Context::try_catch) method, which catches any thrown
//! exception and restores the context to non-throwing state.
//!
//! ## See also
//!
//! 1. Ecma International. [Execution contexts](https://tc39.es/ecma262/#sec-execution-contexts), _ECMAScript Language Specification_.
//! 2. Madhavan Nagarajan. [What is the Execution Context and Stack in JavaScript?](https://medium.com/@itIsMadhavan/what-is-the-execution-context-stack-in-javascript-e169812e851a)
//! 3. Rupesh Mishra. [Execution context, Scope chain and JavaScript internals](https://medium.com/@happymishra66/execution-context-in-javascript-319dd72e8e2c).
//!
//! [lifetime]: https://doc.rust-lang.org/book/ch10-00-generics.html
//! [iterator]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Iterators_and_Generators
//! [question-mark]: https://doc.rust-lang.org/edition-guide/rust-2018/error-handling-and-panics/the-question-mark-operator-for-easier-error-handling.html

pub(crate) mod internal;

use crate::borrow::internal::Ledger;
use crate::borrow::{Borrow, BorrowMut, Ref, RefMut};
use crate::context::internal::Env;
#[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
use crate::event::Channel;
use crate::handle::{Handle, Managed};
#[cfg(feature = "napi-6")]
use crate::lifecycle::InstanceData;
#[cfg(feature = "legacy-runtime")]
use crate::object::class::Class;
use crate::object::{Object, This};
use crate::result::{JsResult, NeonResult, Throw};
use crate::types::binary::{JsArrayBuffer, JsBuffer};
#[cfg(feature = "napi-1")]
use crate::types::boxed::{Finalize, JsBox};
#[cfg(feature = "napi-5")]
use crate::types::date::{DateError, JsDate};
use crate::types::error::JsError;
use crate::types::{
    JsArray, JsBoolean, JsFunction, JsNull, JsNumber, JsObject, JsString, JsUndefined, JsValue,
    StringResult, Value,
};
use neon_runtime;
use neon_runtime::raw;
#[cfg(feature = "napi-1")]
use smallvec::SmallVec;
use std;
use std::cell::RefCell;
use std::convert::Into;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::panic::UnwindSafe;

use self::internal::{ContextInternal, Scope, ScopeMetadata};

#[repr(C)]
pub(crate) struct CallbackInfo<'a> {
    info: raw::FunctionCallbackInfo,
    _lifetime: PhantomData<&'a raw::FunctionCallbackInfo>,
}

impl CallbackInfo<'_> {
    pub fn data(&self, env: Env) -> *mut c_void {
        unsafe {
            let mut raw_data: *mut c_void = std::mem::zeroed();
            neon_runtime::call::data(env.to_raw(), self.info, &mut raw_data);
            raw_data
        }
    }

    pub unsafe fn with_cx<T: This, U, F: for<'a> FnOnce(CallContext<'a, T>) -> U>(
        &self,
        env: Env,
        f: F,
    ) -> U {
        CallContext::<T>::with(env, self, f)
    }

    #[cfg(feature = "legacy-runtime")]
    pub fn set_return<T: Value>(&self, value: Handle<T>) {
        unsafe { neon_runtime::call::set_return(self.info, value.to_raw()) }
    }

    #[cfg(feature = "legacy-runtime")]
    fn kind(&self) -> CallKind {
        if unsafe { neon_runtime::call::is_construct(std::mem::transmute(self)) } {
            CallKind::Construct
        } else {
            CallKind::Call
        }
    }

    #[cfg(feature = "napi-1")]
    fn kind<'b, C: Context<'b>>(&self, cx: &C) -> CallKind {
        if unsafe { neon_runtime::call::is_construct(cx.env().to_raw(), self.info) } {
            CallKind::Construct
        } else {
            CallKind::Call
        }
    }

    pub fn len<'b, C: Context<'b>>(&self, cx: &C) -> i32 {
        unsafe { neon_runtime::call::len(cx.env().to_raw(), self.info) }
    }

    #[cfg(feature = "legacy-runtime")]
    pub fn get<'b, C: Context<'b>>(&self, cx: &mut C, i: i32) -> Option<Handle<'b, JsValue>> {
        if i < 0 || i >= self.len(cx) {
            return None;
        }
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::call::get(cx.env().to_raw(), self.info, i, &mut local);
            Some(Handle::new_internal(JsValue::from_raw(cx.env(), local)))
        }
    }

    #[cfg(feature = "napi-1")]
    pub fn argv<'b, C: Context<'b>>(&self, cx: &mut C) -> SmallVec<[raw::Local; 8]> {
        unsafe { neon_runtime::call::argv(cx.env().to_raw(), self.info) }
    }

    pub fn this<'b, C: Context<'b>>(&self, cx: &mut C) -> raw::Local {
        let env = cx.env();
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::call::this(env.to_raw(), std::mem::transmute(self.info), &mut local);
            local
        }
    }
}

/// Indicates whether a function was called with `new`.
#[derive(Clone, Copy, Debug)]
pub enum CallKind {
    Construct,
    Call,
}

/// A temporary lock of an execution context.
///
/// While a lock is alive, no JavaScript code can be executed in the execution context.
///
/// Objects that support the `Borrow` and `BorrowMut` traits can be inspected while the context is locked by passing a reference to a `Lock` to their methods.
pub struct Lock<'a> {
    pub(crate) ledger: RefCell<Ledger>,
    pub(crate) env: Env,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Lock<'a> {
    fn new(env: Env) -> Self {
        Lock {
            ledger: RefCell::new(Ledger::new()),
            env,
            phantom: PhantomData,
        }
    }
}

/// An _execution context_, which represents the current state of a thread of execution in the JavaScript engine.
///
/// All interaction with the JavaScript engine in Neon code is mediated through instances of this trait.
///
/// A context has a lifetime `'a`, which ensures the safety of handles managed by the JS garbage collector. All handles created during the lifetime of a context are kept alive for that duration and cannot outlive the context.
pub trait Context<'a>: ContextInternal<'a> {
    /// Lock the JavaScript engine, returning an RAII guard that keeps the lock active as long as the guard is alive.
    ///
    /// If this is not the currently active context (for example, if it was used to spawn a scoped context with `execute_scoped` or `compute_scoped`), this method will panic.
    fn lock(&self) -> Lock<'_> {
        self.check_active();
        Lock::new(self.env())
    }

    /// Convenience method for locking the JavaScript engine and borrowing a single JS value's internals.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// # use neon::prelude::*;
    /// # fn my_neon_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    /// let b: Handle<JsArrayBuffer> = cx.argument(0)?;
    /// let x: u32 = cx.borrow(&b, |data| { data.as_slice()[0] });
    /// let n: Handle<JsNumber> = cx.number(x);
    /// # Ok(n)
    /// # }
    /// ```
    ///
    /// Note: the borrowed value is required to be a reference to a handle instead of a handle
    /// as a workaround for a [Rust compiler bug](https://github.com/rust-lang/rust/issues/29997).
    /// We may be able to generalize this compatibly in the future when the Rust bug is fixed,
    /// but while the extra `&` is a small ergonomics regression, this API is still a nice
    /// convenience.
    fn borrow<'c, V, T, F>(&self, v: &'c Handle<V>, f: F) -> T
    where
        V: Value,
        &'c V: Borrow,
        F: for<'b> FnOnce(Ref<'b, <&'c V as Borrow>::Target>) -> T,
    {
        let lock = self.lock();
        let contents = v.borrow(&lock);
        f(contents)
    }

    /// Convenience method for locking the JavaScript engine and mutably borrowing a single JS value's internals.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// # use neon::prelude::*;
    /// # fn my_neon_function(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// let mut b: Handle<JsArrayBuffer> = cx.argument(0)?;
    /// cx.borrow_mut(&mut b, |data| {
    ///     let slice = data.as_mut_slice::<u32>();
    ///     slice[0] += 1;
    /// });
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    ///
    /// Note: the borrowed value is required to be a reference to a handle instead of a handle
    /// as a workaround for a [Rust compiler bug](https://github.com/rust-lang/rust/issues/29997).
    /// We may be able to generalize this compatibly in the future when the Rust bug is fixed,
    /// but while the extra `&mut` is a small ergonomics regression, this API is still a nice
    /// convenience.
    fn borrow_mut<'c, V, T, F>(&self, v: &'c mut Handle<V>, f: F) -> T
    where
        V: Value,
        &'c mut V: BorrowMut,
        F: for<'b> FnOnce(RefMut<'b, <&'c mut V as Borrow>::Target>) -> T,
    {
        let lock = self.lock();
        let contents = v.borrow_mut(&lock);
        f(contents)
    }

    /// Executes a computation in a new memory management scope.
    ///
    /// Handles created in the new scope are kept alive only for the duration of the computation and cannot escape.
    ///
    /// This method can be useful for limiting the life of temporary values created during long-running computations, to prevent leaks.
    fn execute_scoped<T, F>(&self, f: F) -> T
    where
        F: for<'b> FnOnce(ExecuteContext<'b>) -> T,
    {
        self.check_active();
        self.deactivate();
        let result = ExecuteContext::with(self, f);
        self.activate();
        result
    }

    /// Executes a computation in a new memory management scope and computes a single result value that outlives the computation.
    ///
    /// Handles created in the new scope are kept alive only for the duration of the computation and cannot escape, with the exception of the result value, which is rooted in the outer context.
    ///
    /// This method can be useful for limiting the life of temporary values created during long-running computations, to prevent leaks.
    fn compute_scoped<V, F>(&self, f: F) -> JsResult<'a, V>
    where
        V: Value,
        F: for<'b, 'c> FnOnce(ComputeContext<'b, 'c>) -> JsResult<'b, V>,
    {
        self.check_active();
        self.deactivate();
        let result = ComputeContext::with(self, |cx| unsafe {
            let escapable_handle_scope = cx.scope.handle_scope as *mut raw::EscapableHandleScope;
            let escapee = f(cx)?;
            let mut result_local: raw::Local = std::mem::zeroed();
            neon_runtime::scope::escape(
                self.env().to_raw(),
                &mut result_local,
                escapable_handle_scope,
                escapee.to_raw(),
            );
            Ok(Handle::new_internal(V::from_raw(self.env(), result_local)))
        });
        self.activate();
        result
    }

    #[cfg(feature = "try-catch-api")]
    fn try_catch<T, F>(&mut self, f: F) -> Result<T, Handle<'a, JsValue>>
    where
        F: FnOnce(&mut Self) -> NeonResult<T>,
    {
        self.try_catch_internal(f)
    }

    /// Convenience method for creating a `JsBoolean` value.
    fn boolean(&mut self, b: bool) -> Handle<'a, JsBoolean> {
        JsBoolean::new(self, b)
    }

    /// Convenience method for creating a `JsNumber` value.
    fn number<T: Into<f64>>(&mut self, x: T) -> Handle<'a, JsNumber> {
        JsNumber::new(self, x.into())
    }

    /// Convenience method for creating a `JsString` value.
    ///
    /// If the string exceeds the limits of the JS engine, this method panics.
    fn string<S: AsRef<str>>(&mut self, s: S) -> Handle<'a, JsString> {
        JsString::new(self, s)
    }

    /// Convenience method for creating a `JsString` value.
    ///
    /// If the string exceeds the limits of the JS engine, this method returns an `Err` value.
    fn try_string<S: AsRef<str>>(&mut self, s: S) -> StringResult<'a> {
        JsString::try_new(self, s)
    }

    /// Convenience method for creating a `JsNull` value.
    fn null(&mut self) -> Handle<'a, JsNull> {
        #[cfg(feature = "legacy-runtime")]
        return JsNull::new();
        #[cfg(feature = "napi-1")]
        return JsNull::new(self);
    }

    /// Convenience method for creating a `JsUndefined` value.
    fn undefined(&mut self) -> Handle<'a, JsUndefined> {
        #[cfg(feature = "legacy-runtime")]
        return JsUndefined::new();
        #[cfg(feature = "napi-1")]
        return JsUndefined::new(self);
    }

    /// Convenience method for creating an empty `JsObject` value.
    fn empty_object(&mut self) -> Handle<'a, JsObject> {
        JsObject::new(self)
    }

    /// Convenience method for creating an empty `JsArray` value.
    fn empty_array(&mut self) -> Handle<'a, JsArray> {
        JsArray::new(self, 0)
    }

    /// Convenience method for creating an empty `JsArrayBuffer` value.
    fn array_buffer(&mut self, size: u32) -> JsResult<'a, JsArrayBuffer> {
        JsArrayBuffer::new(self, size)
    }

    /// Convenience method for creating an empty `JsBuffer` value.
    fn buffer(&mut self, size: u32) -> JsResult<'a, JsBuffer> {
        JsBuffer::new(self, size)
    }

    /// Convenience method for creating a `JsDate` value.
    #[cfg(feature = "napi-5")]
    fn date(&mut self, value: impl Into<f64>) -> Result<Handle<'a, JsDate>, DateError> {
        JsDate::new(self, value)
    }

    /// Produces a handle to the JavaScript global object.
    fn global(&mut self) -> Handle<'a, JsObject> {
        JsObject::build(|out| unsafe {
            neon_runtime::scope::get_global(self.env().to_raw(), out);
        })
    }

    /// Throws a JS value.
    fn throw<T: Value, U>(&mut self, v: Handle<T>) -> NeonResult<U> {
        unsafe {
            neon_runtime::error::throw(self.env().to_raw(), v.to_raw());
        }
        Err(Throw)
    }

    /// Creates a direct instance of the [`Error`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Error) class.
    fn error<S: AsRef<str>>(&mut self, msg: S) -> JsResult<'a, JsError> {
        JsError::error(self, msg)
    }

    /// Creates an instance of the [`TypeError`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/TypeError) class.
    fn type_error<S: AsRef<str>>(&mut self, msg: S) -> JsResult<'a, JsError> {
        JsError::type_error(self, msg)
    }

    /// Creates an instance of the [`RangeError`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/RangeError) class.
    fn range_error<S: AsRef<str>>(&mut self, msg: S) -> JsResult<'a, JsError> {
        JsError::range_error(self, msg)
    }

    /// Throws a direct instance of the [`Error`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Error) class.
    fn throw_error<S: AsRef<str>, T>(&mut self, msg: S) -> NeonResult<T> {
        let err = JsError::error(self, msg)?;
        self.throw(err)
    }

    /// Throws an instance of the [`TypeError`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/TypeError) class.
    fn throw_type_error<S: AsRef<str>, T>(&mut self, msg: S) -> NeonResult<T> {
        let err = JsError::type_error(self, msg)?;
        self.throw(err)
    }

    /// Throws an instance of the [`RangeError`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/RangeError) class.
    fn throw_range_error<S: AsRef<str>, T>(&mut self, msg: S) -> NeonResult<T> {
        let err = JsError::range_error(self, msg)?;
        self.throw(err)
    }

    #[cfg(feature = "napi-1")]
    /// Convenience method for wrapping a value in a `JsBox`.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use neon::prelude::*;
    /// struct Point(usize, usize);
    ///
    /// impl Finalize for Point {}
    ///
    /// fn my_neon_function(mut cx: FunctionContext) -> JsResult<JsBox<Point>> {
    ///     let point = cx.boxed(Point(0, 1));
    ///
    ///     Ok(point)
    /// }
    /// ```
    fn boxed<U: Finalize + Send + 'static>(&mut self, v: U) -> Handle<'a, JsBox<U>> {
        JsBox::new(self, v)
    }

    #[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
    /// Creates an unbounded channel for scheduling events to be executed on the JavaScript thread.
    ///
    /// When using N-API >= 6,the channel returned by this method is backed by a shared queue.
    /// To create a channel backed by a _new_ queue see [`Channel`](crate::event::Channel).
    fn channel(&mut self) -> Channel {
        #[cfg(feature = "napi-6")]
        let channel = InstanceData::channel(self);

        #[cfg(not(feature = "napi-6"))]
        let channel = Channel::new(self);

        channel
    }

    #[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
    #[deprecated(since = "0.9.0", note = "Please use the channel() method instead")]
    #[doc(hidden)]
    fn queue(&mut self) -> Channel {
        self.channel()
    }
}

/// An execution context of module initialization.
pub struct ModuleContext<'a> {
    scope: Scope<'a, raw::HandleScope>,
    exports: Handle<'a, JsObject>,
}

impl<'a> UnwindSafe for ModuleContext<'a> {}

impl<'a> ModuleContext<'a> {
    pub(crate) fn with<T, F: for<'b> FnOnce(ModuleContext<'b>) -> T>(
        env: Env,
        exports: Handle<'a, JsObject>,
        f: F,
    ) -> T {
        // These assertions ensure the proper amount of space is reserved on the rust stack
        // This is only necessary in the legacy runtime.
        #[cfg(feature = "legacy-runtime")]
        {
            debug_assert!(
                unsafe { neon_runtime::scope::size() } <= std::mem::size_of::<raw::HandleScope>()
            );
            debug_assert!(
                unsafe { neon_runtime::scope::alignment() }
                    <= std::mem::align_of::<raw::HandleScope>()
            );
        }
        Scope::with(env, |scope| f(ModuleContext { scope, exports }))
    }

    /// Convenience method for exporting a Neon function from a module.
    pub fn export_function<T: Value>(
        &mut self,
        key: &str,
        f: fn(FunctionContext) -> JsResult<T>,
    ) -> NeonResult<()> {
        let value = JsFunction::new(self, f)?.upcast::<JsValue>();
        self.exports.set(self, key, value)?;
        Ok(())
    }

    #[cfg(feature = "legacy-runtime")]
    /// Convenience method for exporting a Neon class constructor from a module.
    pub fn export_class<T: Class>(&mut self, key: &str) -> NeonResult<()> {
        let constructor = T::constructor(self)?;
        self.exports.set(self, key, constructor)?;
        Ok(())
    }

    /// Exports a JavaScript value from a Neon module.
    pub fn export_value<T: Value>(&mut self, key: &str, val: Handle<T>) -> NeonResult<()> {
        self.exports.set(self, key, val)?;
        Ok(())
    }

    /// Produces a handle to a module's exports object.
    pub fn exports_object(&mut self) -> JsResult<'a, JsObject> {
        Ok(self.exports)
    }
}

impl<'a> ContextInternal<'a> for ModuleContext<'a> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

impl<'a> Context<'a> for ModuleContext<'a> {}

/// An execution context of a scope created by [`Context::execute_scoped()`](Context::execute_scoped).
pub struct ExecuteContext<'a> {
    scope: Scope<'a, raw::HandleScope>,
}

impl<'a> ExecuteContext<'a> {
    pub(crate) fn with<T, C: Context<'a>, F: for<'b> FnOnce(ExecuteContext<'b>) -> T>(
        cx: &C,
        f: F,
    ) -> T {
        Scope::with(cx.env(), |scope| f(ExecuteContext { scope }))
    }
}

impl<'a> ContextInternal<'a> for ExecuteContext<'a> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

impl<'a> Context<'a> for ExecuteContext<'a> {}

/// An execution context of a scope created by [`Context::compute_scoped()`](Context::compute_scoped).
pub struct ComputeContext<'a, 'outer> {
    scope: Scope<'a, raw::EscapableHandleScope>,
    phantom_inner: PhantomData<&'a ()>,
    phantom_outer: PhantomData<&'outer ()>,
}

impl<'a, 'b> ComputeContext<'a, 'b> {
    pub(crate) fn with<T, C: Context<'a>, F: for<'c, 'd> FnOnce(ComputeContext<'c, 'd>) -> T>(
        cx: &C,
        f: F,
    ) -> T {
        Scope::with(cx.env(), |scope| {
            f(ComputeContext {
                scope,
                phantom_inner: PhantomData,
                phantom_outer: PhantomData,
            })
        })
    }
}

impl<'a, 'b> ContextInternal<'a> for ComputeContext<'a, 'b> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

impl<'a, 'b> Context<'a> for ComputeContext<'a, 'b> {}

/// An execution context of a function call.
///
/// The type parameter `T` is the type of the `this`-binding.
pub struct CallContext<'a, T: This> {
    scope: Scope<'a, raw::HandleScope>,
    info: &'a CallbackInfo<'a>,
    #[cfg(feature = "napi-1")]
    arguments: Option<SmallVec<[raw::Local; 8]>>,
    phantom_type: PhantomData<T>,
}

impl<'a, T: This> UnwindSafe for CallContext<'a, T> {}

impl<'a, T: This> CallContext<'a, T> {
    /// Indicates whether the function was called with `new`.
    pub fn kind(&self) -> CallKind {
        #[cfg(feature = "legacy-runtime")]
        let kind = self.info.kind();

        #[cfg(feature = "napi-1")]
        let kind = self.info.kind(self);

        kind
    }

    pub(crate) fn with<U, F: for<'b> FnOnce(CallContext<'b, T>) -> U>(
        env: Env,
        info: &'a CallbackInfo<'a>,
        f: F,
    ) -> U {
        Scope::with(env, |scope| {
            f(CallContext {
                scope,
                info,
                #[cfg(feature = "napi-1")]
                arguments: None,
                phantom_type: PhantomData,
            })
        })
    }

    /// Indicates the number of arguments that were passed to the function.
    pub fn len(&self) -> i32 {
        self.info.len(self)
    }

    /// Indicates if no arguments were passed to the function.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Produces the `i`th argument, or `None` if `i` is greater than or equal to `self.len()`.
    pub fn argument_opt(&mut self, i: i32) -> Option<Handle<'a, JsValue>> {
        #[cfg(feature = "legacy-runtime")]
        {
            self.info.get(self, i)
        }

        #[cfg(feature = "napi-1")]
        {
            let local = if let Some(arguments) = &self.arguments {
                arguments.get(i as usize).cloned()
            } else {
                let arguments = self.info.argv(self);
                let local = arguments.get(i as usize).cloned();

                self.arguments = Some(arguments);
                local
            };

            local.map(|local| Handle::new_internal(JsValue::from_raw(self.env(), local)))
        }
    }

    /// Produces the `i`th argument and casts it to the type `V`, or throws an exception if `i` is greater than or equal to `self.len()` or cannot be cast to `V`.
    pub fn argument<V: Value>(&mut self, i: i32) -> JsResult<'a, V> {
        match self.argument_opt(i) {
            Some(v) => v.downcast_or_throw(self),
            None => self.throw_type_error("not enough arguments"),
        }
    }

    /// Produces a handle to the `this`-binding.
    pub fn this(&mut self) -> Handle<'a, T> {
        #[cfg(feature = "legacy-runtime")]
        let this = T::as_this(self.info.this(self));
        #[cfg(feature = "napi-1")]
        let this = T::as_this(self.env(), self.info.this(self));

        Handle::new_internal(this)
    }
}

impl<'a, T: This> ContextInternal<'a> for CallContext<'a, T> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

impl<'a, T: This> Context<'a> for CallContext<'a, T> {}

/// A shorthand for a [`CallContext`](CallContext) with `this`-type [`JsObject`](crate::types::JsObject).
pub type FunctionContext<'a> = CallContext<'a, JsObject>;

/// An alias for [`CallContext`](CallContext), useful for indicating that the function is a method of a class.
pub type MethodContext<'a, T> = CallContext<'a, T>;

/// An execution context of a task completion callback.
pub struct TaskContext<'a> {
    /// We use an "inherited HandleScope" here because the C++ `neon::Task::complete`
    /// method sets up and tears down a `HandleScope` for us.
    scope: Scope<'a, raw::InheritedHandleScope>,
}

impl<'a> TaskContext<'a> {
    #[cfg(feature = "legacy-runtime")]
    pub(crate) fn with<T, F: for<'b> FnOnce(TaskContext<'b>) -> T>(f: F) -> T {
        let env = Env::current();
        Scope::with(env, |scope| f(TaskContext { scope }))
    }

    #[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
    pub(crate) fn with_context<T, F: for<'b> FnOnce(TaskContext<'b>) -> T>(env: Env, f: F) -> T {
        Scope::with(env, |scope| f(TaskContext { scope }))
    }
}

impl<'a> ContextInternal<'a> for TaskContext<'a> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

impl<'a> Context<'a> for TaskContext<'a> {}

/// A view of the JS engine in the context of a finalize method on garbage collection
#[cfg(feature = "napi-1")]
pub(crate) struct FinalizeContext<'a> {
    scope: Scope<'a, raw::HandleScope>,
}

#[cfg(feature = "napi-1")]
impl<'a> FinalizeContext<'a> {
    pub(crate) fn with<T, F: for<'b> FnOnce(FinalizeContext<'b>) -> T>(env: Env, f: F) -> T {
        Scope::with(env, |scope| f(FinalizeContext { scope }))
    }
}

#[cfg(feature = "napi-1")]
impl<'a> ContextInternal<'a> for FinalizeContext<'a> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

#[cfg(feature = "napi-1")]
impl<'a> Context<'a> for FinalizeContext<'a> {}
