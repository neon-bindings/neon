use std::{
    any::{self, Any},
    ops::Deref
};

use crate::{
    context::{
        internal::{ContextInternal, Env},
        Context, Cx,
    },
    handle::{internal::TransparentNoCopyWrapper, Handle},
    object::Object,
    sys::{external, raw},
    types::{boxed::private::JsBoxInner, private::ValueInternal, Value},
};

type BoxAny = Box<dyn Any + 'static>;

mod private {
    pub struct JsBoxInner<T: 'static> {
        pub(super) local: crate::sys::raw::Local,
        // Cached raw pointer to the data contained in the `JsBox`. This value is
        // required to implement `Deref` for `JsBox`. Unlike most `Js` types, `JsBox`
        // is not a transparent wrapper around a `napi_value` and cannot implement `This`.
        //
        // Safety: `JsBox` cannot verify the lifetime. Store a raw pointer to force
        // uses to be marked unsafe. In practice, it can be treated as `'static` but
        // should only be exposed as part of a `Handle` tied to a `Context` lifetime.
        // Safety: The value must not move on the heap; we must never give a mutable
        // reference to the data until the `JsBox` is no longer accessible.
        pub(super) raw_data: *const T,
    }
}

/// A JavaScript smart pointer object that owns Rust data.
///
/// The type `JsBox<T>` provides shared ownership of a value of type `T`,
/// allocated in the heap. The data is owned by the JavaScript engine and the
/// lifetime is managed by the JavaScript garbage collector.
///
/// Shared references in Rust disallow mutation by default, and `JsBox` is no
/// exception: you cannot generally obtain a mutable reference to something
/// inside a `JsBox`. If you need to mutate through a `JsBox`, use
/// [`Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html),
/// [`RefCell`](https://doc.rust-lang.org/stable/std/cell/struct.RefCell.html),
/// or one of the other types that provide
/// [interior mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html).
///
/// Values contained by a `JsBox` must implement the `Finalize` trait. `Finalize::finalize`
/// will execute with the value in a `JsBox` immediately before the `JsBox` is garbage
/// collected. If no additional finalization is necessary, an emply implementation may
/// be provided.
///
///
/// ## `Deref` behavior
///
/// `JsBox<T>` automatically dereferences to `T` (via the `Deref` trait), so
/// you can call `T`'s method on a value of type `JsBox<T>`.
///
/// ```rust
/// # use neon::prelude::*;
/// # fn my_neon_function(mut cx: FunctionContext) -> JsResult<JsUndefined> {
/// let vec: Handle<JsBox<Vec<_>>> = cx.boxed(vec![1, 2, 3]);
///
/// println!("Length: {}", vec.len());
/// # Ok(cx.undefined())
/// # }
/// ```
///
/// ## Examples
///
/// Passing some immutable data between Rust and JavaScript.
///
/// ```rust
/// # use neon::prelude::*;
/// # use std::path::{Path, PathBuf};
/// fn create_path(mut cx: FunctionContext) -> JsResult<JsBox<PathBuf>> {
///     let path = cx.argument::<JsString>(0)?.value(&mut cx);
///     let path = Path::new(&path).to_path_buf();
///
///     Ok(cx.boxed(path))
/// }
///
/// fn print_path(mut cx: FunctionContext) -> JsResult<JsUndefined> {
///     let path = cx.argument::<JsBox<PathBuf>>(0)?;
///
///     println!("{}", path.display());
///
///     Ok(cx.undefined())
/// }
/// ```
///
/// Passing a user defined struct wrapped in a `RefCell` for mutability. This
/// pattern is useful for creating classes in JavaScript.
///
/// ```rust
/// # use neon::prelude::*;
/// # use std::cell::RefCell;
///
/// type BoxedPerson = JsBox<RefCell<Person>>;
///
/// struct Person {
///      name: String,
/// }
///
/// impl Finalize for Person {}
///
/// impl Person {
///     pub fn new(name: String) -> Self {
///         Person { name }
///     }
///
///     pub fn set_name(&mut self, name: String) {
///         self.name = name;
///     }
///
///     pub fn greet(&self) -> String {
///         format!("Hello, {}!", self.name)
///     }
/// }
///
/// fn person_new(mut cx: FunctionContext) -> JsResult<BoxedPerson> {
///     let name = cx.argument::<JsString>(0)?.value(&mut cx);
///     let person = RefCell::new(Person::new(name));
///
///     Ok(cx.boxed(person))
/// }
///
/// fn person_set_name(mut cx: FunctionContext) -> JsResult<JsUndefined> {
///     let person = cx.argument::<BoxedPerson>(0)?;
///     let mut person = person.borrow_mut();
///     let name = cx.argument::<JsString>(1)?.value(&mut cx);
///
///     person.set_name(name);
///
///     Ok(cx.undefined())
/// }
///
/// fn person_greet(mut cx: FunctionContext) -> JsResult<JsString> {
///     let person = cx.argument::<BoxedPerson>(0)?;
///     let person = person.borrow();
///     let greeting = person.greet();
///
///     Ok(cx.string(greeting))
/// }
#[repr(transparent)]
pub struct JsBox<T: 'static>(JsBoxInner<T>);

impl<T: 'static> std::fmt::Debug for JsBoxInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JsBox<{}>", std::any::type_name::<T>())
    }
}

impl<T: 'static> std::fmt::Debug for JsBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

// Attempt to use a `napi_value` as a `napi_external` to unwrap a `BoxAny>
/// Safety: `local` must be a `napi_value` that is valid for the lifetime `'a`.
unsafe fn maybe_external_deref<'a>(env: Env, local: raw::Local) -> Option<&'a BoxAny> {
    external::deref::<BoxAny>(env.to_raw(), local).map(|v| &*v)
}

// Custom `Clone` implementation since `T` might not be `Clone`
impl<T: 'static> Clone for JsBoxInner<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Object for JsBox<T> {}

impl<T: 'static> Copy for JsBoxInner<T> {}

impl<T: 'static> Value for JsBox<T> {}

unsafe impl<T: 'static> TransparentNoCopyWrapper for JsBox<T> {
    type Inner = JsBoxInner<T>;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl<T: 'static> ValueInternal for JsBox<T> {
    fn name() -> &'static str {
        any::type_name::<Self>()
    }

    fn is_typeof<Other: Value>(cx: &mut Cx, other: &Other) -> bool {
        let data = unsafe { maybe_external_deref(cx.env(), other.to_local()) };

        data.map(|v| v.is::<T>()).unwrap_or(false)
    }

    fn downcast<Other: Value>(cx: &mut Cx, other: &Other) -> Option<Self> {
        let local = other.to_local();
        let data = unsafe { maybe_external_deref(cx.env(), local) };

        // Attempt to downcast the `Option<&BoxAny>` to `Option<*const T>`
        data.and_then(|v| v.downcast_ref())
            .map(|raw_data| Self(JsBoxInner { local, raw_data }))
    }

    fn to_local(&self) -> raw::Local {
        self.0.local
    }

    unsafe fn from_local(env: Env, local: raw::Local) -> Self {
        let raw_data = unsafe { maybe_external_deref(env, local) }
            .expect("Failed to unwrap napi_external as Box<Any>")
            .downcast_ref()
            .expect("Failed to downcast Any");

        Self(JsBoxInner { local, raw_data })
    }
}

/// Values contained by a `JsBox` must be `Finalize + 'static`
///
/// ### `Finalize`
///
/// The `sys::prelude::Finalize` trait provides a `finalize` method that will be called
/// immediately before the `JsBox` is garbage collected.
///
/// ### `'static'
///
/// The lifetime of a `JsBox` is managed by the JavaScript garbage collector. Since Rust
/// is unable to verify the lifetime of the contents, references must be valid for the
/// entire duration of the program. This does not mean that the `JsBox` will be valid
/// until the application terminates, only that its lifetime is indefinite.
impl<T: Finalize + 'static> JsBox<T> {
    /// Constructs a new `JsBox` containing `value`.
    pub fn new<'a, C>(cx: &mut C, value: T) -> Handle<'a, JsBox<T>>
    where
        C: Context<'a>,
        T: 'static,
    {
        // This function will execute immediately before the `JsBox` is garbage collected.
        // It unwraps the `napi_external`, downcasts the `BoxAny` and moves the type
        // out of the `Box`. Lastly, it calls the trait method `Finalize::fianlize` of the
        // contained value `T`.
        fn finalizer<U: Finalize + 'static>(env: raw::Env, data: BoxAny) {
            let data = *data.downcast::<U>().unwrap();
            let env = Env::from(env);

            Cx::with_context(env, move |mut cx| data.finalize(&mut cx));
        }

        let v = Box::new(value) as BoxAny;
        // Since this value was just constructed, we know it is `T`
        let raw_data = &*v as *const dyn Any as *const T;
        let local = unsafe { external::create(cx.env().to_raw(), v, finalizer::<T>) };

        Handle::new_internal(Self(JsBoxInner { local, raw_data }))
    }
}

impl<T: 'static> JsBox<T> {
    pub(crate) fn manually_finalize<'a, C>(cx: &mut C, value: T) -> Handle<'a, JsBox<T>>
    where
        C: Context<'a>,
        T: 'static,
    {
        fn finalizer(_env: raw::Env, _data: BoxAny) {}

        let v = Box::new(value) as BoxAny;
        // Since this value was just constructed, we know it is `T`
        let raw_data = &*v as *const dyn Any as *const T;
        let local = unsafe { external::create(cx.env().to_raw(), v, finalizer) };

        Handle::new_internal(Self(JsBoxInner { local, raw_data }))
    }
}

impl<T: 'static> JsBox<T> {
    /// Gets a reference to the inner value of a [`JsBox`]. This method is similar to
    /// [dereferencing](JsBox::deref) a `JsBox` (e.g., `&*boxed`), but the lifetime
    /// is _safely_ extended to `'cx`.
    ///
    /// See also [`Handle<JsBox>::as_inner`].
    // N.B.: This would be cleaner with https://github.com/rust-lang/rust/issues/44874
    pub fn deref<'cx>(v: &Handle<'cx, Self>) -> &'cx T {
        v.as_inner()
    }
}

impl<'cx, T: 'static> Handle<'cx, JsBox<T>> {
    /// Gets a reference to the inner value of a [`JsBox`]. This method is similar to
    /// [dereferencing](JsBox::deref) a `JsBox` (e.g., `&*boxed`), but the lifetime
    /// is _safely_ extended to `'cx`.
    ///
    /// See also [`JsBox::deref`].
    pub fn as_inner(&self) -> &'cx T {
        // # Safety
        // JS values associated with an in-scope `Context` *cannot* be garbage collected.
        // This value is guaranteed to live at least as long as `'cx`.
        unsafe { &*self.0.raw_data }
    }
}

impl<T: 'static> Deref for JsBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // # Safety
        // `T` will live at least as long as `JsBox<T>` because it may not be garbage
        // collected while in scope and only immutable references can be obtained.
        unsafe { &*self.0.raw_data }
    }
}

/// A trait for finalizing values owned by the main JavaScript thread.
///
/// [`Finalize::finalize`] is executed on the main JavaScript thread
/// immediately before garbage collection.
///
/// Values contained by a `JsBox` must implement `Finalize`.
///
/// ## Examples
///
/// `Finalize` provides a default implementation that does not perform any finalization.
///
/// ```rust
/// # use neon::prelude::*;
/// struct Point(f64, f64);
///
/// impl Finalize for Point {}
/// ```
///
/// A `finalize` method may be specified for performing clean-up operations before dropping
/// the contained value.
///
/// ```rust
/// # use neon::prelude::*;
/// struct Point(f64, f64);
///
/// impl Finalize for Point {
///     fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
///         cx.global_object()
///             .method(cx.cx_mut(), "emit").unwrap()
///             .args(("gc_point", self.0, self.1)).unwrap()
///             .exec().unwrap();
///     }
/// }
/// ```
pub trait Finalize: Sized {
    fn finalize<'a, C: Context<'a>>(self, _: &mut C) {}
}

// Primitives

impl Finalize for bool {}

impl Finalize for char {}

impl Finalize for i8 {}

impl Finalize for i16 {}

impl Finalize for i32 {}

impl Finalize for i64 {}

impl Finalize for isize {}

impl Finalize for u8 {}

impl Finalize for u16 {}

impl Finalize for u32 {}

impl Finalize for u64 {}

impl Finalize for usize {}

impl Finalize for f32 {}

impl Finalize for f64 {}

// Common types

impl Finalize for String {}

impl Finalize for std::path::PathBuf {}

// Tuples

macro_rules! finalize_tuple_impls {
    ($( $name:ident )+) => {
        impl<$($name: Finalize),+> Finalize for ($($name,)+) {
            fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
                #![allow(non_snake_case)]
                let ($($name,)+) = self;
                ($($name.finalize(cx),)+);
            }
        }
    };
}

impl Finalize for () {}
finalize_tuple_impls! { T0 }
finalize_tuple_impls! { T0 T1 }
finalize_tuple_impls! { T0 T1 T2 }
finalize_tuple_impls! { T0 T1 T2 T3 }
finalize_tuple_impls! { T0 T1 T2 T3 T4 }
finalize_tuple_impls! { T0 T1 T2 T3 T4 T5 }
finalize_tuple_impls! { T0 T1 T2 T3 T4 T5 T6 }
finalize_tuple_impls! { T0 T1 T2 T3 T4 T5 T6 T7 }

// Collections

impl<T: Finalize> Finalize for Vec<T> {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        for item in self {
            item.finalize(cx);
        }
    }
}

// Smart pointers and other wrappers

impl<T: Finalize> Finalize for std::boxed::Box<T> {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        (*self).finalize(cx);
    }
}

impl<T: Finalize> Finalize for Option<T> {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        if let Some(v) = self {
            v.finalize(cx);
        }
    }
}

impl<T: Finalize> Finalize for std::rc::Rc<T> {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        if let Ok(v) = std::rc::Rc::try_unwrap(self) {
            v.finalize(cx);
        }
    }
}

impl<T: Finalize> Finalize for std::sync::Arc<T> {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        if let Ok(v) = std::sync::Arc::try_unwrap(self) {
            v.finalize(cx);
        }
    }
}

impl<T: Finalize> Finalize for std::sync::Mutex<T> {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        if let Ok(v) = self.into_inner() {
            v.finalize(cx);
        }
    }
}

impl<T: Finalize> Finalize for std::sync::RwLock<T> {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        if let Ok(v) = self.into_inner() {
            v.finalize(cx);
        }
    }
}

impl<T: Finalize> Finalize for std::cell::Cell<T> {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        self.into_inner().finalize(cx);
    }
}

impl<T: Finalize> Finalize for std::cell::RefCell<T> {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        self.into_inner().finalize(cx);
    }
}
