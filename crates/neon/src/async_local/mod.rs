mod executor;
mod runtime;
mod waker;

use std::future::Future;

use executor::ThreadNotifyRef;
use waker::LocalWaker;
use waker::WakerEvent;

use self::runtime::LocalRuntime;
use crate::context::Context;
use crate::sys;

/// Schedule a future to run asynchronously on the local JavaScript thread.
/// The future's execution will not block the local thread.
pub fn spawn_async_local<'a>(
    cx: &mut impl Context<'a>,
    future: impl Future<Output = ()> + 'static,
) -> Result<(), ()> {
    // Add a future to the future pool to be executed
    // whenever the Nodejs event loop is free to do so
    LocalRuntime::queue_future(future).unwrap();

    // If there are tasks in flight then the executor
    // is already running and should be reused
    if LocalRuntime::futures_count() > 1 {
        return Ok(());
    }

    // The futures executor runs on another thread and will
    // use a threadsafe function to call schedule work
    // on the JavaScript thread
    let env_raw = cx.env().to_raw();

    LocalWaker::send(WakerEvent::Init(unsafe {
        sys::tsfn::ThreadsafeFunction::<ThreadNotifyRef>::new(env_raw, |_, thread_notify| {
            let done = LocalRuntime::run_until_stalled(thread_notify);

            if done {
                LocalWaker::send(WakerEvent::Done);
            } else {
                LocalWaker::send(WakerEvent::Next);
            }
        })
    }));

    Ok(())
}
