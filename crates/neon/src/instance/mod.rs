//! Instance-local storage.
//!
//! At runtime, an instance of a Node.js addon can contain its own local storage,
//! which can then be shared and accessed as needed between Rust modules. This can
//! be useful for setting up long-lived state that needs to be shared between calls
//! of an addon's APIs.
//!
//! For example, an addon that makes use of a
//! [Tokio runtime](https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html)
//! can store the runtime in a [`Local`](Local) cell:
//!
//! ```
//! # use neon::prelude::*;
//! # use neon::instance::Local;
//! # struct Runtime;
//! # type Result<T> = std::result::Result<T, ()>;
//! # impl Runtime {fn new() -> Result<Self> { Err(()) }}
//! static RUNTIME: Local<Runtime> = Local::new();
//!
//! pub fn runtime<'cx, C: Context<'cx>>(cx: &mut C) -> Result<&'cx Runtime> {
//!     RUNTIME.get_or_try_init(cx, |_| Runtime::new())
//! }
//! ```
//!
//! Because Node.js supports [worker threads](https://nodejs.org/api/worker_threads.html),
//! a Node.js addon may have multiple instances in a running process. Local
//! storage is instantiated separately for each instance of the addon.

use std::any::Any;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use once_cell::sync::OnceCell;

use crate::context::Context;
use crate::lifecycle::LocalCell;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn next_id() -> usize {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// A cell that can be used to allocate data that is local to an instance
/// of a Neon addon.
#[derive(Default)]
pub struct Local<T> {
    _type: PhantomData<T>,
    id: OnceCell<usize>,
}

impl<T> Local<T> {
    /// Creates a new local value. This method is `const`, so it can be assigned to
    /// static variables.
    pub const fn new() -> Self {
        Self {
            _type: PhantomData,
            id: OnceCell::new(),
        }
    }

    fn id(&self) -> usize {
        *self.id.get_or_init(next_id)
    }
}

impl<T: Any + Send + 'static> Local<T> {
    /// Gets the current value of the cell. Returns `None` if the cell has not
    /// yet been initialized.
    pub fn get<'cx, 'a, C>(&self, cx: &'a mut C) -> Option<&'cx T>
    where
        C: Context<'cx>,
    {
        // Unwrap safety: The type bound Local<T> and the fact that every Local has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: Option<&T> =
            LocalCell::get(cx, self.id()).map(|value| value.downcast_ref().unwrap());

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        unsafe { std::mem::transmute::<Option<&'a T>, Option<&'cx T>>(r) }
    }

    /// Gets the current value of the cell, initializing it with `value` if it has
    /// not yet been initialized.
    pub fn get_or_init<'cx, 'a, C>(&self, cx: &'a mut C, value: T) -> &'cx T
    where
        C: Context<'cx>,
    {
        // Unwrap safety: The type bound Local<T> and the fact that every Local has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: &T = LocalCell::get_or_init(cx, self.id(), Box::new(value))
            .downcast_ref()
            .unwrap();

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        unsafe { std::mem::transmute::<&'a T, &'cx T>(r) }
    }

    /// Gets the current value of the cell, initializing it with the result of
    /// calling `f` if it has not yet been initialized.
    pub fn get_or_init_with<'cx, 'a, C, F>(&self, cx: &'a mut C, f: F) -> &'cx T
    where
        C: Context<'cx>,
        F: FnOnce() -> T,
    {
        // Unwrap safety: The type bound Local<T> and the fact that every Local has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: &T = LocalCell::get_or_init_with(cx, self.id(), || Box::new(f()))
            .downcast_ref()
            .unwrap();

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        unsafe { std::mem::transmute::<&'a T, &'cx T>(r) }
    }

    /// Gets the current value of the cell, initializing it with the result of
    /// calling `f` if it has not yet been initialized. Returns `Err` if the
    /// callback triggers a JavaScript exception.
    ///
    /// During the execution of `f`, calling any methods on this `Local` that
    /// attempt to initialize it will panic.
    pub fn get_or_try_init<'cx, 'a, C, E, F>(&self, cx: &'a mut C, f: F) -> Result<&'cx T, E>
    where
        C: Context<'cx>,
        F: FnOnce(&mut C) -> Result<T, E>,
    {
        // Unwrap safety: The type bound Local<T> and the fact that every Local has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: &T = LocalCell::get_or_try_init(cx, self.id(), |cx| Ok(Box::new(f(cx)?)))?
            .downcast_ref()
            .unwrap();

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        Ok(unsafe { std::mem::transmute::<&'a T, &'cx T>(r) })
    }
}

impl<T: Any + Send + Default + 'static> Local<T> {
    /// Gets the current value of the cell, initializing it with the default value
    /// if it has not yet been initialized.
    pub fn get_or_init_default<'cx, 'a, C>(&self, cx: &'a mut C) -> &'cx T
    where
        C: Context<'cx>,
    {
        self.get_or_init_with(cx, Default::default)
    }
}
