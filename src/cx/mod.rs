//! Abstractions representing the JavaScript virtual machine and its control flow.

use std;
use std::cell::RefCell;
use std::any::TypeId;
use std::convert::Into;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::panic::UnwindSafe;
use neon_runtime;
use neon_runtime::raw;
use borrow::{Ref, RefMut, Borrow, BorrowMut};
use borrow::internal::Ledger;
use value::{JsResult, JsValue, Value, JsObject, JsArray, JsFunction, JsBoolean, JsNumber, JsString, StringResult, JsNull, JsUndefined};
use value::mem::{Managed, Handle};
use value::binary::{JsArrayBuffer, JsBuffer};
use value::error::{JsError, ErrorKind};
use object::{Object, This};
use object::class::Class;
use object::class::internal::ClassMetadata;
use result::{NeonResult, Throw, ResultExt};
use self::internal::{ContextInternal, Scope, ScopeMetadata};

pub(crate) mod internal {
    use std;
    use std::cell::Cell;
    use std::os::raw::c_void;
    use neon_runtime;
    use neon_runtime::raw;
    use neon_runtime::scope::Root;
    use value::mem::Handle;
    use result::NeonResult;
    use value::JsObject;
    use super::{ClassMap, ModuleContext};

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Isolate(*mut raw::Isolate);

    extern "C" fn drop_class_map(map: Box<ClassMap>) {
        std::mem::drop(map);
    }

    impl Isolate {
        pub(crate) fn to_raw(self) -> *mut raw::Isolate {
            let Isolate(ptr) = self;
            ptr
        }

        pub(crate) fn class_map(&mut self) -> &mut ClassMap {
            let mut ptr: *mut c_void = unsafe { neon_runtime::class::get_class_map(self.to_raw()) };
            if ptr.is_null() {
                let b: Box<ClassMap> = Box::new(ClassMap::new());
                let raw = Box::into_raw(b);
                ptr = unsafe { std::mem::transmute(raw) };
                let free_map: *mut c_void = unsafe { std::mem::transmute(drop_class_map as usize) };
                unsafe {
                    neon_runtime::class::set_class_map(self.to_raw(), ptr, free_map);
                }
            }
            unsafe { std::mem::transmute(ptr) }
        }

        pub(crate) fn current() -> Isolate {
            unsafe {
                std::mem::transmute(neon_runtime::call::current_isolate())
            }
        }
    }

    pub struct ScopeMetadata {
        isolate: Isolate,
        active: Cell<bool>
    }

    pub struct Scope<'a, R: Root + 'static> {
        pub metadata: ScopeMetadata,
        pub handle_scope: &'a mut R
    }

    impl<'a, R: Root + 'static> Scope<'a, R> {
        pub fn with<T, F: for<'b> FnOnce(Scope<'b, R>) -> T>(f: F) -> T {
            let mut handle_scope: R = unsafe { R::allocate() };
            let isolate = Isolate::current();
            unsafe {
                handle_scope.enter(isolate.to_raw());
            }
            let result = {
                let scope = Scope {
                    metadata: ScopeMetadata {
                        isolate,
                        active: Cell::new(true)
                    },
                    handle_scope: &mut handle_scope
                };
                f(scope)
            };
            unsafe {
                handle_scope.exit();
            }
            result
        }
    }

    pub trait ContextInternal<'a>: Sized {
        fn scope_metadata(&self) -> &ScopeMetadata;

        fn isolate(&self) -> Isolate {
            self.scope_metadata().isolate
        }

        fn is_active(&self) -> bool {
            self.scope_metadata().active.get()
        }

        fn check_active(&self) {
            if !self.is_active() {
                panic!("VM context is inactive");
            }
        }

        fn activate(&self) { self.scope_metadata().active.set(true); }
        fn deactivate(&self) { self.scope_metadata().active.set(false); }
    }

    pub fn initialize_module(exports: Handle<JsObject>, init: fn(ModuleContext) -> NeonResult<()>) {
        ModuleContext::with(exports, |cx| {
            let _ = init(cx);
        });
    }
}

pub(crate) struct ClassMap {
    map: HashMap<TypeId, ClassMetadata>
}

impl ClassMap {
    fn new() -> ClassMap {
        ClassMap {
            map: HashMap::new()
        }
    }

    pub fn get(&self, key: &TypeId) -> Option<&ClassMetadata> {
        self.map.get(key)
    }

    pub fn set(&mut self, key: TypeId, val: ClassMetadata) {
        self.map.insert(key, val);
    }
}

#[repr(C)]
pub(crate) struct CallbackInfo {
    info: raw::FunctionCallbackInfo
}

impl CallbackInfo {
    pub fn data<'a>(&self) -> Handle<'a, JsValue> {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::call::data(&self.info, &mut local);
            Handle::new_internal(JsValue::from_raw(local))
        }
    }

    pub unsafe fn with_cx<T: This, U, F: for<'a> FnOnce(CallContext<'a, T>) -> U>(&self, f: F) -> U {
        CallContext::<T>::with(self, f)
    }

    pub fn set_return<'a, 'b, T: Value>(&'a self, value: Handle<'b, T>) {
        unsafe {
            neon_runtime::call::set_return(&self.info, value.to_raw())
        }
    }

    fn kind(&self) -> CallKind {
        if unsafe { neon_runtime::call::is_construct(std::mem::transmute(self)) } {
            CallKind::Construct
        } else {
            CallKind::Call
        }
    }

    pub fn len(&self) -> i32 {
        unsafe {
            neon_runtime::call::len(&self.info)
        }
    }

    pub fn get<'b, C: Context<'b>>(&self, _: &mut C, i: i32) -> Option<Handle<'b, JsValue>> {
        if i < 0 || i >= self.len() {
            return None;
        }
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::call::get(&self.info, i, &mut local);
            Some(Handle::new_internal(JsValue::from_raw(local)))
        }
    }

    pub fn require<'b, C: Context<'b>>(&self, cx: &mut C, i: i32) -> JsResult<'b, JsValue> {
        if i < 0 || i >= self.len() {
            return JsError::throw(cx, ErrorKind::TypeError, "not enough arguments");
        }
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::call::get(&self.info, i, &mut local);
            Ok(Handle::new_internal(JsValue::from_raw(local)))
        }
    }

    pub fn this<'b, V: Context<'b>>(&self, _: &mut V) -> raw::Local {
        unsafe {
            let mut local: raw::Local = std::mem::zeroed();
            neon_runtime::call::this(std::mem::transmute(&self.info), &mut local);
            local
        }
    }
}

/// Indicates whether a function call was called with JavaScript's `[[Call]]` or `[[Construct]]` semantics.
#[derive(Clone, Copy, Debug)]
pub enum CallKind {
    Construct,
    Call
}

/// An RAII implementation of a "scoped lock" of the JS VM. When this structure is dropped (falls out of scope), the VM will be unlocked.
///
/// Types of JS values that support the `Borrow` and `BorrowMut` traits can be inspected while the VM is locked by passing a reference to a `Lock` to their methods.
pub struct Lock<'a> {
    pub(crate) ledger: RefCell<Ledger>,
    phantom: PhantomData<&'a ()>
}

impl<'a> Lock<'a> {
    fn new() -> Self {
        Lock {
            ledger: RefCell::new(Ledger::new()),
            phantom: PhantomData
        }
    }
}

/// A contextual view of the JS VM. Most operations that interact with the VM require passing a reference to a VM context.
/// 
/// A VM context has a lifetime `'a`, which tracks the rooting of handles managed by the JS garbage collector. All handles created during the lifetime of a context are rooted for that duration and cannot outlive the context.
pub trait Context<'a>: ContextInternal<'a> {

    /// Lock the JS VM, returning an RAII guard that keeps the lock active as long as the guard is alive.
    /// 
    /// If this is not the currently active context (for example, if it was used to spawn a scoped context with `execute_scoped` or `compute_scoped`), this method will panic.
    fn lock(&self) -> Lock {
        self.check_active();
        Lock::new()
    }

    /// Convenience method for locking the VM and borrowing a single JS value's internals.
    /// 
    /// # Example:
    /// 
    /// ```no_run
    /// use neon::value::{JsNumber, Borrow, Ref, JsArrayBuffer};
    /// # use neon::vm::{JsResult, FunctionContext};
    /// use neon::vm::{Context, Handle};
    /// 
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
        where V: Value,
              &'c V: Borrow,
              F: for<'b> FnOnce(Ref<'b, <&'c V as Borrow>::Target>) -> T
    {
        let lock = self.lock();
        let contents = v.borrow(&lock);
        f(contents)
    }

    /// Convenience method for locking the VM and mutably borrowing a single JS value's internals.
    /// 
    /// # Example:
    /// 
    /// ```no_run
    /// use neon::value::{BorrowMut, RefMut, JsArrayBuffer};
    /// # use neon::value::JsUndefined;
    /// # use neon::vm::{JsResult, FunctionContext};
    /// use neon::vm::{Context, Handle};
    /// 
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
        where V: Value,
              &'c mut V: BorrowMut,
              F: for<'b> FnOnce(RefMut<'b, <&'c mut V as Borrow>::Target>) -> T
    {
        let lock = self.lock();
        let contents = v.borrow_mut(&lock);
        f(contents)
    }

    /// Executes a computation in a new memory management scope.
    /// 
    /// Handles created in the new scope are rooted for the duration of the computation and cannot escape.
    /// 
    /// This method can be useful for limiting the life of temporary values created during long-running computations, to prevent leaks.
    fn execute_scoped<T, F>(&self, f: F) -> T
        where F: for<'b> FnOnce(ExecuteContext<'b>) -> T
    {
        self.check_active();
        self.deactivate();
        let result = ExecuteContext::with(f);
        self.activate();
        result
    }

    /// Executes a computation in a new memory management scope and computes a single result value that outlives the computation.
    /// 
    /// Handles created in the new scope are rooted for the duration of the computation and cannot escape, with the exception of the result value, which is rooted in the current context.
    /// 
    /// This method can be useful for limiting the life of temporary values created during long-running computations, to prevent leaks.
    fn compute_scoped<V, F>(&self, f: F) -> JsResult<'a, V>
        where V: Value,
              F: for<'b, 'c> FnOnce(ComputeContext<'b, 'c>) -> JsResult<'b, V>
    {
        self.check_active();
        self.deactivate();
        let result = ComputeContext::with(|cx| {
            unsafe {
                let escapable_handle_scope = cx.scope.handle_scope as *mut raw::EscapableHandleScope;
                let escapee = f(cx)?;
                let mut result_local: raw::Local = std::mem::zeroed();
                neon_runtime::scope::escape(&mut result_local, escapable_handle_scope, escapee.to_raw());
                Ok(Handle::new_internal(V::from_raw(result_local)))
            }
        });
        self.activate();
        result
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
    /// If the string exceeds the limits of the JS VM, this method panics.
    fn string<S: AsRef<str>>(&mut self, s: S) -> Handle<'a, JsString> {
        JsString::new(self, s)
    }

    /// Convenience method for creating a `JsString` value.
    /// 
    /// If the string exceeds the limits of the JS VM, this method returns an `Err` value.
    fn try_string<S: AsRef<str>>(&mut self, s: S) -> StringResult<'a> {
        JsString::try_new(self, s)
    }

    /// Convenience method for creating a `JsNull` value.
    fn null(&mut self) -> Handle<'a, JsNull> {
        JsNull::new()
    }

    /// Convenience method for creating a `JsUndefined` value.
    fn undefined(&mut self) -> Handle<'a, JsUndefined> {
        JsUndefined::new()
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

    /// Produces a handle to the JavaScript global object.
    fn global(&mut self) -> Handle<'a, JsObject> {
        JsObject::build(|out| {
            unsafe {
                neon_runtime::scope::get_global(self.isolate().to_raw(), out);
            }
        })
    }

    /// Throws a JS value.
    fn throw<'b, T: Value, U>(&mut self, v: Handle<'b, T>) -> NeonResult<U> {
        unsafe {
            neon_runtime::error::throw(v.to_raw());
        }
        Err(Throw)
    }
}

/// A view of the JS VM in the context of top-level initialization of a Neon module.
pub struct ModuleContext<'a> {
    scope: Scope<'a, raw::HandleScope>,
    exports: Handle<'a, JsObject>
}

impl<'a> UnwindSafe for ModuleContext<'a> { }

impl<'a> ModuleContext<'a> {
    pub(crate) fn with<T, F: for<'b> FnOnce(ModuleContext<'b>) -> T>(exports: Handle<'a, JsObject>, f: F) -> T {
        debug_assert!(unsafe { neon_runtime::scope::size() } <= std::mem::size_of::<raw::HandleScope>());
        debug_assert!(unsafe { neon_runtime::scope::alignment() } <= std::mem::align_of::<raw::HandleScope>());
        Scope::with(|scope| {
            f(ModuleContext {
                scope,
                exports
            })
        })
    }

    /// Convenience method for exporting a Neon function from a module.
    pub fn export_function<T: Value>(&mut self, key: &str, f: fn(FunctionContext) -> JsResult<T>) -> NeonResult<()> {
        let value = JsFunction::new(self, f)?.upcast::<JsValue>();
        self.exports.set(self, key, value)?;
        Ok(())
    }

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

impl<'a> Context<'a> for ModuleContext<'a> { }

/// A view of the JS VM in the context of a scoped computation started by `Context::execute_scoped()`.
pub struct ExecuteContext<'a> {
    scope: Scope<'a, raw::HandleScope>
}

impl<'a> ExecuteContext<'a> {
    pub(crate) fn with<T, F: for<'b> FnOnce(ExecuteContext<'b>) -> T>(f: F) -> T {
        Scope::with(|scope| {
            f(ExecuteContext { scope })
        })
    }
}

impl<'a> ContextInternal<'a> for ExecuteContext<'a> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

impl<'a> Context<'a> for ExecuteContext<'a> { }

/// A view of the JS VM in the context of a scoped computation started by `Context::compute_scoped()`.
pub struct ComputeContext<'a, 'outer> {
    scope: Scope<'a, raw::EscapableHandleScope>,
    phantom_inner: PhantomData<&'a ()>,
    phantom_outer: PhantomData<&'outer ()>
}

impl<'a, 'b> ComputeContext<'a, 'b> {
    pub(crate) fn with<T, F: for<'c, 'd> FnOnce(ComputeContext<'c, 'd>) -> T>(f: F) -> T {
        Scope::with(|scope| {
            f(ComputeContext {
                scope,
                phantom_inner: PhantomData,
                phantom_outer: PhantomData
            })
        })
    }
}

impl<'a, 'b> ContextInternal<'a> for ComputeContext<'a, 'b> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

impl<'a, 'b> Context<'a> for ComputeContext<'a, 'b> { }

/// A view of the JS VM in the context of a function call.
/// 
/// The type parameter `T` is the type of the `this`-binding.
pub struct CallContext<'a, T: This> {
    scope: Scope<'a, raw::HandleScope>,
    info: &'a CallbackInfo,
    phantom_type: PhantomData<T>
}

impl<'a, T: This> UnwindSafe for CallContext<'a, T> { }

impl<'a, T: This> CallContext<'a, T> {
    /// Indicates whether the function was called via the JavaScript `[[Call]]` or `[[Construct]]` semantics.
    pub fn kind(&self) -> CallKind { self.info.kind() }

    pub(crate) fn with<U, F: for<'b> FnOnce(CallContext<'b, T>) -> U>(info: &'a CallbackInfo, f: F) -> U {
        Scope::with(|scope| {
            f(CallContext {
                scope,
                info,
                phantom_type: PhantomData
            })
        })
    }

    /// Indicates the number of arguments that were passed to the function.
    pub fn len(&self) -> i32 { self.info.len() }

    /// Produces the `i`th argument, or `None` if `i` is greater than or equal to `self.len()`.
    pub fn argument_opt(&mut self, i: i32) -> Option<Handle<'a, JsValue>> {
        self.info.get(self, i)
    }

    /// Produces the `i`th argument and casts it to the type `V`, or throws an exception if `i` is greater than or equal to `self.len()` or cannot be cast to `V`.
    pub fn argument<V: Value>(&mut self, i: i32) -> JsResult<'a, V> {
        let a = self.info.require(self, i)?;
        a.downcast().unwrap_or_throw(self)
    }

    /// Produces a handle to the `this`-binding.
    pub fn this(&mut self) -> Handle<'a, T> {
        Handle::new_internal(T::as_this(self.info.this(self)))
    }
}

impl<'a, T: This> ContextInternal<'a> for CallContext<'a, T> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

impl<'a, T: This> Context<'a> for CallContext<'a, T> { }

/// A shorthand for a `CallContext` with `this`-type `JsObject`.
pub type FunctionContext<'a> = CallContext<'a, JsObject>;

/// An alias for `CallContext`, useful for indicating that the function is a method of a class.
pub type MethodContext<'a, T> = CallContext<'a, T>;

/// A view of the JS VM in the context of a task completion callback.
pub struct TaskContext<'a> {
    /// We use an "inherited HandleScope" here because the C++ `neon::Task::complete`
    /// method sets up and tears down a `HandleScope` for us.
    scope: Scope<'a, raw::InheritedHandleScope>
}

impl<'a> TaskContext<'a> {
    pub(crate) fn with<T, F: for<'b> FnOnce(TaskContext<'b>) -> T>(f: F) -> T {
        Scope::with(|scope| {
            f(TaskContext { scope })
        })
    }
}

impl<'a> ContextInternal<'a> for TaskContext<'a> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

impl<'a> Context<'a> for TaskContext<'a> { }
