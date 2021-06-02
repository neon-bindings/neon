//! # Environment life cycle APIs
//!
//! These APIs map to the life cycle of a specific "Agent" or self-contained
//! environment. If a Neon module is loaded multiple times (Web Workers, worker
//! threads), these API will be handle data associated with a specific instance.
//!
//! See the [N-API Lifecycle][npai-docs] documentation for more details.
//!
//! [napi-docs]: https://nodejs.org/api/n-api.html#n_api_environment_life_cycle_apis

use std::mem;
use std::sync::Arc;

use neon_runtime::raw::Env;
use neon_runtime::reference;
use neon_runtime::tsfn::ThreadsafeFunction;

use crate::context::Context;
use crate::event::Channel;
use crate::handle::root::NapiRef;

/// `InstanceData` holds Neon data associated with a particular instance of a
/// native module. If a module is loaded multiple times (e.g., worker threads), this
/// data will be unique per instance.
pub(crate) struct InstanceData {
    /// Used to free `Root` in the same JavaScript environment that created it
    ///
    /// _Design Note_: An `Arc` ensures the `ThreadsafeFunction` outlives the unloading
    /// of a module. Since it is unlikely that modules will be re-loaded frequently, this
    /// could be replaced with a leaked `&'static ThreadsafeFunction<NapiRef>`. However,
    /// given the cost of FFI, this optimization is omitted until the cost of an
    /// `Arc` is demonstrated as significant.
    drop_queue: Arc<ThreadsafeFunction<NapiRef>>,

    /// Shared `Channel` that is cloned to be returned by the `cx.channel()` method
    shared_channel: Channel,
}

fn drop_napi_ref(env: Option<Env>, data: NapiRef) {
    if let Some(env) = env {
        unsafe {
            reference::unreference(env, mem::transmute(data));
        }
    }
}

impl InstanceData {
    /// Return the data associated with this module instance, lazily initializing if
    /// necessary.
    ///
    /// # Safety
    /// No additional locking (e.g., `Mutex`) is necessary because holding a
    /// `Context` reference ensures serialized access.
    pub(crate) fn get<'a, C: Context<'a>>(cx: &mut C) -> &'a mut InstanceData {
        let env = cx.env().to_raw();
        let data =
            unsafe { neon_runtime::lifecycle::get_instance_data::<InstanceData>(env).as_mut() };

        if let Some(data) = data {
            return data;
        }

        let drop_queue = unsafe {
            let mut queue = ThreadsafeFunction::new(env, drop_napi_ref);
            queue.unref(env).unwrap();
            queue
        };

        let shared_channel = {
            let mut channel = Channel::new(cx);
            channel.unref(cx);
            channel
        };

        let data = InstanceData {
            drop_queue: Arc::new(drop_queue),
            shared_channel,
        };

        unsafe { &mut *neon_runtime::lifecycle::set_instance_data(env, data) }
    }

    /// Helper to return a reference to the `drop_queue` field of `InstanceData`
    pub(crate) fn drop_queue<'a, C: Context<'a>>(cx: &mut C) -> Arc<ThreadsafeFunction<NapiRef>> {
        Arc::clone(&InstanceData::get(cx).drop_queue)
    }

    /// Clones the shared channel and references it since new channels should start
    /// referenced, but the shared channel is unreferenced.
    pub(crate) fn channel<'a, C: Context<'a>>(cx: &mut C) -> Channel {
        let mut channel = InstanceData::get(cx).shared_channel.clone();
        channel.reference(cx);
        channel
    }
}
