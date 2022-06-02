use std::any::Any;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use once_cell::sync::OnceCell;

use crate::context::Context;
use crate::lifecycle::GlobalCell;
use crate::result::NeonResult;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn next_id() -> usize {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// A cell that can be used to allocate data that is global to an instance
/// of a Neon addon.
#[derive(Default)]
pub struct Global<T> {
    _type: PhantomData<T>,
    id: OnceCell<usize>,
}

impl<T> Global<T> {
    /// Creates a new global value. This method is `const`, so it can be assigned to
    /// static variables.
    pub const fn new() -> Self {
        Default::default()
    }

    fn id(&self) -> usize {
        *self.id.get_or_init(next_id)
    }
}

impl<T: Any + Send + 'static> Global<T> {
    /// Gets the current value of the global. Returns `None` if the global has not
    /// yet been initialized.
    pub fn get<'cx, 'a, C>(&self, cx: &'a mut C) -> Option<&'cx T>
    where
        C: Context<'cx>,
    {
        // Safety: The type bound Global<T> and the fact that every Global has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: Option<&T> =
            GlobalCell::get(cx, self.id()).map(|value| value.downcast_ref().unwrap());

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        unsafe { std::mem::transmute::<Option<&'a T>, Option<&'cx T>>(r) }
    }

    /// Gets the current value of the global, initializing it with `value` if it has
    /// not yet been initialized.
    pub fn get_or_init<'cx, 'a, C>(&self, cx: &'a mut C, value: T) -> &'cx T
    where
        C: Context<'cx>,
    {
        // Safety: The type bound Global<T> and the fact that every Global has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: &T = GlobalCell::get_or_init(cx, self.id(), Box::new(value))
            .downcast_ref()
            .unwrap();

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        unsafe { std::mem::transmute::<&'a T, &'cx T>(r) }
    }

    /// Gets the current value of the global, initializing it with the result of
    /// calling `f` if it has not yet been initialized.
    pub fn get_or_init_with<'cx, 'a, C, F>(&self, cx: &'a mut C, f: F) -> &'cx T
    where
        C: Context<'cx>,
        F: FnOnce() -> T,
    {
        // Safety: The type bound Global<T> and the fact that every Global has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: &T = GlobalCell::get_or_init_with(cx, self.id(), || Box::new(f()))
            .downcast_ref()
            .unwrap();

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        unsafe { std::mem::transmute::<&'a T, &'cx T>(r) }
    }

    /// Gets the current value of the global, initializing it with the result of
    /// calling `f` if it has not yet been initialized. Returns `Err` if the
    /// callback triggers a JavaScript exception.
    ///
    /// During the execution of `f`, calling any methods on this `Global` that
    /// attempt to initialize it will panic.
    pub fn get_or_try_init<'cx, 'a, C, F>(&self, cx: &'a mut C, f: F) -> NeonResult<&'cx T>
    where
        C: Context<'cx>,
        F: FnOnce(&mut C) -> NeonResult<T>,
    {
        // Safety: The type bound Global<T> and the fact that every Global has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: &T = GlobalCell::get_or_try_init(cx, self.id(), |cx| Ok(Box::new(f(cx)?)))?
            .downcast_ref()
            .unwrap();

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        Ok(unsafe { std::mem::transmute::<&'a T, &'cx T>(r) })
    }
}

impl<T: Any + Send + Default + 'static> Global<T> {
    /// Gets the current value of the global, initializing it with the default value
    /// if it has not yet been initialized.
    pub fn get_or_init_default<'cx, 'a, C>(&self, cx: &'a mut C) -> &'cx T
    where
        C: Context<'cx>,
    {
        self.get_or_init_with(cx, Default::default)
    }
}
