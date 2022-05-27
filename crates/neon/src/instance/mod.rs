use std::any::Any;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use once_cell::sync::OnceCell;

use crate::context::Context;
use crate::lifecycle::InstanceData;
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
            std::mem::transmute::<Option<&'a T>, Option<&'cx T>>(r)
        }
    }

    pub fn get_or_init<'cx, 'a, C>(&self, cx: &'a mut C, value: T) -> &'cx T
    where
        C: Context<'cx>,
    {
        let r: &T = InstanceData::globals(cx)
            .get(self.id())
            .get_or_insert(Box::new(value))
            .downcast_ref()
            .unwrap();

        unsafe {
            std::mem::transmute::<&'a T, &'cx T>(r)
        }
    }

    pub fn get_or_init_with<'cx, 'a, C, F>(&self, cx: &'a mut C, f: F) -> &'cx T
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

    pub fn get_or_try_init<'cx, 'a, C, F>(&self, cx: &'a mut C, f: F) -> NeonResult<&'cx T>
    where
        C: Context<'cx>,
        F: FnOnce(&mut C) -> NeonResult<T>,
    {
        if let Some(value) = self.get(cx) {
            return Ok(value);
        }

        let value = f(cx)?;

        if self.get(cx).is_some() {
            panic!("Global already initialized during get_or_try_init callback");
        }

        let r: &T = InstanceData::globals(cx).get(self.id())
            .insert(Box::new(value))
            .downcast_ref()
            .unwrap();
        
        Ok(unsafe {
            std::mem::transmute::<&'a T, &'cx T>(r)
        })
    }
}

impl<T: Any + Send + Default + 'static> Global<T> {
    pub fn get_or_init_default<'cx, 'a, C>(&self, cx: &'a mut C) -> &'cx T
    where
        C: Context<'cx>,
    {
        self.get_or_init_with(cx, Default::default)
    }
}
