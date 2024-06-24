mod executor;
mod runtime;

use std::future::Future;

use self::runtime::LocalFuturesCount;
use self::runtime::LocalRuntime;
use crate::context::Context;
use crate::sys;
use crate::types::JsFunction;

pub(crate) fn spawn_async_local<'a>(
    cx: &mut impl Context<'a>,
    future: impl Future + 'static,
) -> Result<(), ()> {
    let future = async move {
        future.await;
        LocalFuturesCount::decrement();
    };

    LocalRuntime::spawn_local(future).unwrap();

    // If there are tasks in flight then the
    // executor is already initialized
    if LocalFuturesCount::count() != 0 {
        return Ok(());
    }

    // Start the futures digest cycle
    let env_raw = cx.env().to_raw();

    unsafe { sys::fun::new(env_raw, "", |env, _| {

    }) };

    // unsafe {
    //     sys::create_threadsafe_function(
    //         env_raw,
    //         func,
    //         async_resource,
    //         async_resource_name,
    //         max_queue_size,
    //         initial_thread_count,
    //         thread_finalize_data,
    //         thread_finalize_cb,
    //         context,
    //         call_js_cb,
    //         result,
    //     )
    // }

    Ok(())
}
