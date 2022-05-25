use std::any::Any;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use once_cell::sync::OnceCell;

use crate::context::Context;
use crate::lifecycle::InstanceData;

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

impl<T: Any + Send + 'static> Global<T> {
    pub fn get<'cx, 'a, C>(&self, cx: &'a mut C) -> Option<&'cx T>
    where
        C: Context<'cx>,
    {
        let r: Option<&T> = InstanceData::globals(cx)
            .get(self.id())
            .as_ref()
            .map(|boxed| boxed.downcast_ref().unwrap());

        unsafe {
            std::mem::transmute(r)
        }
    }

    pub fn get_or_init<'cx, 'a, C, F>(&self, cx: &'a mut C, f: F) -> &'cx T
    where
        C: Context<'cx>,
        F: FnOnce() -> T,
    {
        let r: &T = InstanceData::globals(cx)
            .get(self.id())
            .get_or_insert_with(|| Box::new(f()))
            .downcast_ref()
            .unwrap();

        unsafe {
            std::mem::transmute::<&'a T, &'cx T>(r)
        }
    }
}
