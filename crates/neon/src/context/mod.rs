//! Provides runtime access to the JavaScript engine.
//!
//! An _execution context_ represents the current state of a thread of execution in the
//! JavaScript engine. Internally, it tracks things like the set of pending function calls,
//! whether the engine is currently throwing an exception or not, and whether the engine is
//! in the process of shutting down. The context uses this internal state to manage what
//! operations are safely available and when.
//!
//! The [`Context`] trait provides an abstract interface to the JavaScript
//! execution context. All interaction with the JavaScript engine in Neon code is mediated
//! through instances of this trait.
//!
//! One particularly useful context type is [`FunctionContext`], which is passed
//! to all Neon functions as their initial execution context.
//!
//! ```
//! # use neon::prelude::*;
//! fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
//!     Ok(cx.string("hello Neon"))
//! }
//! ```
//!
//! Another important context type is [`ModuleContext`], which is provided
//! to a Neon module's [`main`](crate::main) function to enable sharing Neon functions back
//! with JavaScript:
//!
//! ```
//! # use neon::prelude::*;
//! # fn hello(_: FunctionContext) -> JsResult<JsValue> { todo!() }
//! #[neon::main]
//! fn lib(mut cx: ModuleContext) -> NeonResult<()> {
//!     cx.export_function("hello", hello)?;
//!     Ok(())
//! }
//! ```
//!
//! ## Writing Generic Helpers
//!
//! Depending on the entrypoint, a user may have a [`FunctionContext`], [`ModuleContext`], or
//! generic [`Cx`]. While it is possible to write a helper that is generic over the [`Context`]
//! trait, it is often simpler to accept a [`Cx`] argument. Due to deref coercion, other contexts
//! may be passed into a function that accepts a reference to [`Cx`].
//!
//! ```
//! # use neon::prelude::*;
//! fn log(cx: &mut Cx, msg: &str) -> NeonResult<()> {
//!     cx.global::<JsObject>("console")?
//!         .call_method_with(cx, "log")?
//!         .arg(cx.string(msg))
//!         .exec(cx)?;
//!
//!     Ok(())
//! }
//!
//! fn print(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//!     let msg = cx.argument::<JsString>(0)?.value(&mut cx);
//!     log(&mut cx, &msg)?;
//!     Ok(cx.undefined())
//! }
//! ```
//!
//! ## Memory Management
//!
//! Because contexts represent the engine at a point in time, they are associated with a
//! [_lifetime_][lifetime], which limits how long Rust code is allowed to access them. This
//! is also used to determine the lifetime of [`Handle`]s, which
//! provide safe references to JavaScript memory managed by the engine's garbage collector.
//!
//! For example, we can
//! write a simple string scanner that counts whitespace in a JavaScript string and
//! returns a [`JsNumber`]:
//!
//! ```
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
//! # use neon::prelude::*;
//! # fn iterate(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//!     let iterator = cx.argument::<JsObject>(0)?;         // iterator object
//!     let next: Handle<JsFunction> =                      // iterator's `next` method
//!         iterator.prop(&mut cx, "next").get()?;
//!     let mut numbers = vec![];                           // results vector
//!     let mut done = false;                               // loop controller
//!
//!     while !done {
//!         done = cx.execute_scoped(|mut cx| {                   // temporary scope
//!             let obj: Handle<JsObject> = next                  // temporary object
//!                 .call_with(&cx)
//!                 .this(iterator)
//!                 .apply(&mut cx)?;
//!             numbers.push(obj.prop(&mut cx, "value").get()?);  // temporary number
//!             obj.prop(&mut cx, "done").get()                   // temporary boolean
//!         })?;
//!     }
//! #   Ok(cx.undefined())
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
//! [`Err`] result, indicating that the thread associated
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
//! [`Context::try_catch`] method, which catches any thrown
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

use std::{
    convert::Into,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    panic::UnwindSafe,
};

pub use crate::types::buffer::lock::Lock;

use crate::{
    event::TaskBuilder,
    handle::Handle,
    object::Object,
    result::{JsResult, NeonResult, Throw},
    sys::{
        self, raw,
        scope::{EscapableHandleScope, HandleScope},
    },
    types::{
        boxed::{Finalize, JsBox},
        error::JsError,
        extract::FromArgs,
        private::ValueInternal,
        Deferred, JsArray, JsArrayBuffer, JsBoolean, JsBuffer, JsFunction, JsNull, JsNumber,
        JsObject, JsPromise, JsString, JsUndefined, JsValue, StringResult, Value,
    },
};

use self::internal::{ContextInternal, Env};

#[cfg(feature = "napi-4")]
use crate::event::Channel;

#[cfg(feature = "napi-5")]
use crate::types::date::{DateError, JsDate};

#[cfg(feature = "napi-6")]
use crate::lifecycle::InstanceData;

#[doc(hidden)]
/// An execution context of a task completion callback.
pub type TaskContext<'cx> = Cx<'cx>;

#[doc(hidden)]
/// An execution context of a scope created by [`Context::execute_scoped()`](Context::execute_scoped).
pub type ExecuteContext<'cx> = Cx<'cx>;

#[doc(hidden)]
/// An execution context of a scope created by [`Context::compute_scoped()`](Context::compute_scoped).
pub type ComputeContext<'cx> = Cx<'cx>;

#[doc(hidden)]
/// A view of the JS engine in the context of a finalize method on garbage collection
pub type FinalizeContext<'cx> = Cx<'cx>;

/// An execution context constructed from a raw [`Env`](crate::sys::bindings::Env).
#[cfg(feature = "sys")]
#[cfg_attr(docsrs, doc(cfg(feature = "sys")))]
#[doc(hidden)]
pub type SysContext<'cx> = Cx<'cx>;

/// Context representing access to the JavaScript runtime
pub struct Cx<'cx> {
    env: Env,
    _phantom_inner: PhantomData<&'cx ()>,
}

impl<'cx> Cx<'cx> {
    /// Creates a context from a raw `Env`.
    ///
    /// # Safety
    ///
    /// Once a [`Cx`] has been created, it is unsafe to use
    /// the `Env`. The handle scope for the `Env` must be valid for
    /// the lifetime `'cx`.
    #[cfg(feature = "sys")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sys")))]
    pub unsafe fn from_raw(env: sys::Env) -> Self {
        Self {
            env: env.into(),
            _phantom_inner: PhantomData,
        }
    }

    fn new(env: Env) -> Self {
        Self {
            env,
            _phantom_inner: PhantomData,
        }
    }

    pub(crate) fn with_context<T, F: for<'b> FnOnce(Cx<'b>) -> T>(env: Env, f: F) -> T {
        f(Self {
            env,
            _phantom_inner: PhantomData,
        })
    }
}

impl<'cx> ContextInternal<'cx> for Cx<'cx> {
    fn cx(&self) -> &Cx<'cx> {
        self
    }

    fn cx_mut(&mut self) -> &mut Cx<'cx> {
        self
    }
}

impl<'cx> Context<'cx> for Cx<'cx> {}

impl<'cx> From<FunctionContext<'cx>> for Cx<'cx> {
    fn from(cx: FunctionContext<'cx>) -> Self {
        cx.cx
    }
}

impl<'cx> From<ModuleContext<'cx>> for Cx<'cx> {
    fn from(cx: ModuleContext<'cx>) -> Self {
        cx.cx
    }
}

#[repr(C)]
pub(crate) struct CallbackInfo<'cx> {
    info: raw::FunctionCallbackInfo,
    _lifetime: PhantomData<&'cx raw::FunctionCallbackInfo>,
}

impl CallbackInfo<'_> {
    pub unsafe fn new(info: raw::FunctionCallbackInfo) -> Self {
        Self {
            info,
            _lifetime: PhantomData,
        }
    }

    fn kind<'b, C: Context<'b>>(&self, cx: &C) -> CallKind {
        if unsafe { sys::call::is_construct(cx.env().to_raw(), self.info) } {
            CallKind::Construct
        } else {
            CallKind::Call
        }
    }

    pub fn len<'b, C: Context<'b>>(&self, cx: &C) -> usize {
        unsafe { sys::call::len(cx.env().to_raw(), self.info) }
    }

    pub fn argv<'b, C: Context<'b>>(&self, cx: &mut C) -> sys::call::Arguments {
        unsafe { sys::call::argv(cx.env().to_raw(), self.info) }
    }

    pub fn this<'b, C: Context<'b>>(&self, cx: &mut C) -> raw::Local {
        let env = cx.env();
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            sys::call::this(env.to_raw(), self.info, &mut local);
            local
        }
    }

    pub(crate) fn argv_exact<'b, C: Context<'b>, const N: usize>(
        &self,
        cx: &mut C,
    ) -> [Handle<'b, JsValue>; N] {
        use std::ptr;

        let mut argv = [JsValue::new_internal(ptr::null_mut()); N];
        let mut argc = argv.len();

        // # Safety
        // * Node-API fills empty slots with `undefined`
        // * `Handle` and `JsValue` are transparent wrappers around a raw pointer
        unsafe {
            assert_eq!(
                sys::get_cb_info(
                    cx.env().to_raw(),
                    self.info,
                    &mut argc,
                    argv.as_mut_ptr().cast(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                ),
                sys::Status::Ok,
            );
        }

        // Empty values will be filled with `undefined`
        argv
    }
}

/// Indicates whether a function was called with `new`.
#[derive(Clone, Copy, Debug)]
pub enum CallKind {
    Construct,
    Call,
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
    fn lock<'b>(&'b mut self) -> Lock<'b, Self>
    where
        'a: 'b,
    {
        Lock::new(self)
    }

    /// Executes a computation in a new memory management scope.
    ///
    /// Handles created in the new scope are kept alive only for the duration of the computation and cannot escape.
    ///
    /// This method can be useful for limiting the life of temporary values created during long-running computations, to prevent leaks.
    fn execute_scoped<'b, T, F>(&mut self, f: F) -> T
    where
        'a: 'b,
        F: FnOnce(Cx<'b>) -> T,
    {
        let env = self.env();
        let scope = unsafe { HandleScope::new(env.to_raw()) };
        let result = f(Cx::new(env));

        drop(scope);

        result
    }

    /// Executes a computation in a new memory management scope and computes a single result value that outlives the computation.
    ///
    /// Handles created in the new scope are kept alive only for the duration of the computation and cannot escape, with the exception of the result value, which is rooted in the outer context.
    ///
    /// This method can be useful for limiting the life of temporary values created during long-running computations, to prevent leaks.
    fn compute_scoped<'b, V, F>(&mut self, f: F) -> JsResult<'a, V>
    where
        'a: 'b,
        V: Value,
        F: FnOnce(Cx<'b>) -> JsResult<'b, V>,
    {
        let env = self.env();
        let scope = unsafe { EscapableHandleScope::new(env.to_raw()) };
        let cx = Cx::new(env);

        let escapee = unsafe { scope.escape(f(cx)?.to_local()) };

        Ok(Handle::new_internal(unsafe {
            V::from_local(self.env(), escapee)
        }))
    }

    fn try_catch<T, F>(&mut self, f: F) -> Result<T, Handle<'a, JsValue>>
    where
        F: FnOnce(&mut Self) -> NeonResult<T>,
    {
        unsafe {
            self.env()
                .try_catch(move || f(self))
                .map_err(JsValue::new_internal)
        }
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
        JsNull::new(self)
    }

    /// Convenience method for creating a `JsUndefined` value.
    fn undefined(&mut self) -> Handle<'a, JsUndefined> {
        JsUndefined::new(self)
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
    fn array_buffer(&mut self, size: usize) -> JsResult<'a, JsArrayBuffer> {
        JsArrayBuffer::new(self, size)
    }

    /// Convenience method for creating an empty `JsBuffer` value.
    fn buffer(&mut self, size: usize) -> JsResult<'a, JsBuffer> {
        JsBuffer::new(self, size)
    }
    /// Convenience method for creating a `JsDate` value.
    #[cfg(feature = "napi-5")]
    #[cfg_attr(docsrs, doc(cfg(feature = "napi-5")))]
    fn date(&mut self, value: impl Into<f64>) -> Result<Handle<'a, JsDate>, DateError> {
        JsDate::new(self, value)
    }

    /// Convenience method for looking up a global property by name.
    ///
    /// Equivalent to:
    ///
    /// ```
    /// # use neon::prelude::*;
    /// # fn get_array_global<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction> {
    /// #     let name = "Array";
    /// #     let v: Handle<JsFunction> =
    /// {
    ///     let global = cx.global_object();
    ///     global.prop(cx, name).get()
    /// }
    /// #     ?;
    /// #     Ok(v)
    /// # }
    /// ```
    fn global<T: Value>(&mut self, name: &str) -> JsResult<'a, T> {
        let global = self.global_object();
        global.get(self, name)
    }

    /// Produces a handle to the JavaScript global object.
    fn global_object(&mut self) -> Handle<'a, JsObject> {
        JsObject::build(|out| unsafe {
            sys::scope::get_global(self.env().to_raw(), out);
        })
    }

    /// Throws a JS value.
    fn throw<T: Value, U>(&mut self, v: Handle<T>) -> NeonResult<U> {
        unsafe {
            sys::error::throw(self.env().to_raw(), v.to_local());
            Err(Throw::new())
        }
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
    fn boxed<U: Finalize + 'static>(&mut self, v: U) -> Handle<'a, JsBox<U>> {
        JsBox::new(self, v)
    }

    #[cfg(feature = "napi-4")]
    #[deprecated(since = "0.9.0", note = "Please use the channel() method instead")]
    #[doc(hidden)]
    fn queue(&mut self) -> Channel {
        self.channel()
    }

    #[cfg(feature = "napi-4")]
    #[cfg_attr(docsrs, doc(cfg(feature = "napi-4")))]
    /// Returns an unbounded channel for scheduling events to be executed on the JavaScript thread.
    ///
    /// When using N-API >= 6,the channel returned by this method is backed by a shared queue.
    /// To create a channel backed by a _new_ queue see [`Channel`].
    fn channel(&mut self) -> Channel {
        #[cfg(feature = "napi-6")]
        let channel = InstanceData::channel(self);

        #[cfg(not(feature = "napi-6"))]
        let channel = Channel::new(self);

        channel
    }

    /// Creates a [`Deferred`] and [`JsPromise`] pair. The [`Deferred`] handle can be
    /// used to resolve or reject the [`JsPromise`].
    ///
    /// ```
    /// # use neon::prelude::*;
    /// fn resolve_promise(mut cx: FunctionContext) -> JsResult<JsPromise> {
    ///     let (deferred, promise) = cx.promise();
    ///     let msg = cx.string("Hello, World!");
    ///
    ///     deferred.resolve(&mut cx, msg);
    ///
    ///     Ok(promise)
    /// }
    /// ```
    fn promise(&mut self) -> (Deferred, Handle<'a, JsPromise>) {
        JsPromise::new(self)
    }

    /// Creates a [`TaskBuilder`] which can be used to schedule the `execute`
    /// callback to asynchronously execute on the
    /// [Node worker pool](https://nodejs.org/en/docs/guides/dont-block-the-event-loop/).
    ///
    /// ```
    /// # use neon::prelude::*;
    /// fn greet(mut cx: FunctionContext) -> JsResult<JsPromise> {
    ///     let name = cx.argument::<JsString>(0)?.value(&mut cx);
    ///
    ///     let promise = cx
    ///         .task(move || format!("Hello, {}!", name))
    ///         .promise(move |mut cx, greeting| Ok(cx.string(greeting)));
    ///
    ///     Ok(promise)
    /// }
    /// ```
    fn task<'cx, O, E>(&'cx mut self, execute: E) -> TaskBuilder<'cx, Self, E>
    where
        'a: 'cx,
        O: Send + 'static,
        E: FnOnce() -> O + Send + 'static,
    {
        TaskBuilder::new(self, execute)
    }

    #[cfg(feature = "sys")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sys")))]
    /// Gets the raw `sys::Env` for usage with Node-API.
    fn to_raw(&self) -> sys::Env {
        self.env().to_raw()
    }
}

/// An execution context of module initialization.
pub struct ModuleContext<'cx> {
    cx: Cx<'cx>,
    exports: Handle<'cx, JsObject>,
}

impl<'cx> Deref for ModuleContext<'cx> {
    type Target = Cx<'cx>;

    fn deref(&self) -> &Self::Target {
        self.cx()
    }
}

impl<'cx> DerefMut for ModuleContext<'cx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.cx_mut()
    }
}

impl<'cx> UnwindSafe for ModuleContext<'cx> {}

impl<'cx> ModuleContext<'cx> {
    pub(crate) fn with<T, F: for<'b> FnOnce(ModuleContext<'b>) -> T>(
        env: Env,
        exports: Handle<'cx, JsObject>,
        f: F,
    ) -> T {
        f(ModuleContext {
            cx: Cx::new(env),
            exports,
        })
    }

    #[cfg(not(feature = "napi-5"))]
    /// Convenience method for exporting a Neon function from a module.
    pub fn export_function<T: Value>(
        &mut self,
        key: &str,
        f: fn(FunctionContext) -> JsResult<T>,
    ) -> NeonResult<()> {
        let value = JsFunction::new(self, f)?.upcast::<JsValue>();
        self.exports.clone().set(self, key, value)?;
        Ok(())
    }

    #[cfg(feature = "napi-5")]
    /// Convenience method for exporting a Neon function from a module.
    pub fn export_function<F, V>(&mut self, key: &str, f: F) -> NeonResult<()>
    where
        F: Fn(FunctionContext) -> JsResult<V> + 'static,
        V: Value,
    {
        let value = JsFunction::new(self, f)?.upcast::<JsValue>();
        // Note: Cloning `exports` is necessary to avoid holding a shared reference to
        // `self` while attempting to use it mutably in `set`.
        self.exports.clone().set(self, key, value)?;
        Ok(())
    }

    /// Exports a JavaScript value from a Neon module.
    pub fn export_value<T: Value>(&mut self, key: &str, val: Handle<T>) -> NeonResult<()> {
        self.exports.clone().set(self, key, val)?;
        Ok(())
    }

    /// Produces a handle to a module's exports object.
    pub fn exports_object(&mut self) -> JsResult<'cx, JsObject> {
        Ok(self.exports)
    }
}

impl<'cx> ContextInternal<'cx> for ModuleContext<'cx> {
    fn cx(&self) -> &Cx<'cx> {
        &self.cx
    }

    fn cx_mut(&mut self) -> &mut Cx<'cx> {
        &mut self.cx
    }
}

impl<'cx> Context<'cx> for ModuleContext<'cx> {}

/// An execution context of a function call.
///
/// The type parameter `T` is the type of the `this`-binding.
pub struct FunctionContext<'cx> {
    cx: Cx<'cx>,
    info: &'cx CallbackInfo<'cx>,

    arguments: Option<sys::call::Arguments>,
}

impl<'cx> Deref for FunctionContext<'cx> {
    type Target = Cx<'cx>;

    fn deref(&self) -> &Self::Target {
        &self.cx
    }
}

impl<'cx> DerefMut for FunctionContext<'cx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cx
    }
}

impl<'cx> UnwindSafe for FunctionContext<'cx> {}

impl<'cx> FunctionContext<'cx> {
    /// Indicates whether the function was called with `new`.
    pub fn kind(&self) -> CallKind {
        self.info.kind(self)
    }

    pub(crate) fn with<U, F: for<'b> FnOnce(FunctionContext<'b>) -> U>(
        env: Env,
        info: &'cx CallbackInfo<'cx>,
        f: F,
    ) -> U {
        f(FunctionContext {
            cx: Cx::new(env),
            info,
            arguments: None,
        })
    }

    /// Indicates the number of arguments that were passed to the function.
    pub fn len(&self) -> usize {
        self.info.len(self)
    }

    /// Indicates if no arguments were passed to the function.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Produces the `i`th argument, or `None` if `i` is greater than or equal to `self.len()`.
    pub fn argument_opt(&mut self, i: usize) -> Option<Handle<'cx, JsValue>> {
        let argv = if let Some(argv) = self.arguments.as_ref() {
            argv
        } else {
            let argv = self.info.argv(self);
            self.arguments.insert(argv)
        };

        argv.get(i)
            .map(|v| Handle::new_internal(unsafe { JsValue::from_local(self.env(), v) }))
    }

    /// Produces the `i`th argument and casts it to the type `V`, or throws an exception if `i` is greater than or equal to `self.len()` or cannot be cast to `V`.
    pub fn argument<V: Value>(&mut self, i: usize) -> JsResult<'cx, V> {
        match self.argument_opt(i) {
            Some(v) => v.downcast_or_throw(self),
            None => self.throw_type_error("not enough arguments"),
        }
    }

    /// Produces a handle to the `this`-binding and attempts to downcast as a specific type.
    /// Equivalent to calling `cx.this_value().downcast_or_throw(&mut cx)`.
    ///
    /// Throws an exception if the value is a different type.
    pub fn this<T: Value>(&mut self) -> JsResult<'cx, T> {
        self.this_value().downcast_or_throw(self)
    }

    /// Produces a handle to the function's [`this`-binding](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/this#function_context).
    pub fn this_value(&mut self) -> Handle<'cx, JsValue> {
        JsValue::new_internal(self.info.this(self))
    }

    /// Extract Rust data from the JavaScript arguments.
    ///
    /// This is frequently more efficient and ergonomic than getting arguments
    /// individually. See the [`extract`](crate::types::extract) module documentation
    /// for more examples.
    ///
    /// ```
    /// # use neon::{prelude::*, types::extract::*};
    /// fn add(mut cx: FunctionContext) -> JsResult<JsNumber> {
    ///     let (a, b): (f64, f64) = cx.args()?;
    ///
    ///     Ok(cx.number(a + b))
    /// }
    /// ```
    pub fn args<T>(&mut self) -> NeonResult<T>
    where
        T: FromArgs<'cx>,
    {
        T::from_args(self)
    }

    /// Extract Rust data from the JavaScript arguments.
    ///
    /// Similar to [`FunctionContext::args`], but does not throw a JavaScript exception on errors. Useful
    /// for function overloading.
    ///
    /// ```
    /// # use neon::{prelude::*, types::extract::*};
    /// fn combine(mut cx: FunctionContext) -> JsResult<JsValue> {
    ///     if let Some((a, b)) = cx.args_opt::<(f64, f64)>()? {
    ///         return Ok(cx.number(a + b).upcast());
    ///     }
    ///
    ///     let (a, b): (String, String) = cx.args()?;
    ///
    ///     Ok(cx.string(a + &b).upcast())
    /// }
    /// ```
    pub fn args_opt<T>(&mut self) -> NeonResult<Option<T>>
    where
        T: FromArgs<'cx>,
    {
        T::from_args_opt(self)
    }

    pub(crate) fn argv<const N: usize>(&mut self) -> [Handle<'cx, JsValue>; N] {
        self.info.argv_exact(self)
    }
}

impl<'cx> ContextInternal<'cx> for FunctionContext<'cx> {
    fn cx(&self) -> &Cx<'cx> {
        &self.cx
    }

    fn cx_mut(&mut self) -> &mut Cx<'cx> {
        &mut self.cx
    }
}

impl<'cx> Context<'cx> for FunctionContext<'cx> {}
