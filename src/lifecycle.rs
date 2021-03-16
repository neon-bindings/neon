use std::mem;

use neon_runtime::raw::Env;
use neon_runtime::reference;
use neon_runtime::tsfn::ThreadsafeFunction;

use crate::context::Context;
use crate::handle::root::NapiRef;

pub(crate) struct InstanceData {
    drop_queue: &'static ThreadsafeFunction<NapiRef>,
}

fn drop_napi_ref(env: Option<Env>, data: NapiRef) {
    if let Some(env) = env {
        unsafe {
            reference::unreference(env, mem::transmute(data));
        }
    }
}

impl InstanceData {
    pub(crate) fn get<'a, C: Context<'a>>(cx: &mut C) -> &'a mut InstanceData {
        let env = cx.env().to_raw();
        let data =
            unsafe { neon_runtime::lifecycle::get_instance_data::<InstanceData>(env).as_mut() };

        if let Some(data) = data {
            return data;
        }

        let drop_queue = Box::new(unsafe {
            let mut queue = ThreadsafeFunction::new(env, drop_napi_ref);
            queue.unref(env);
            queue
        });

        let data = InstanceData {
            drop_queue: Box::leak(drop_queue),
        };

        unsafe { &mut *neon_runtime::lifecycle::set_instance_data(env, data) }
    }

    pub(crate) fn drop_queue<'a, C: Context<'a>>(
        cx: &mut C,
    ) -> &'static ThreadsafeFunction<NapiRef> {
        &InstanceData::get(cx).drop_queue
    }
}
