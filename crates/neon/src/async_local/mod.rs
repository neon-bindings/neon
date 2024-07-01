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
) {
    // Add a future to the future pool to be executed
    // whenever the Nodejs event loop is free to do so
    LocalRuntime::queue_future(future);

    // If there are tasks in flight then the executor
    // is already running and should be reused
    if LocalRuntime::futures_count() > 1 {
        return;
    }

    // The futures executor runs on the main thread thread but
    // the waker runs on another thread.
    //
    // The main thread executor will run the contained futures
    // and as soon as they stall (e.g. waiting for a channel, timer, etc),
    // the executor will immediately yield back to the JavaScript event loop.
    //
    // This "parks" the executer, which normally means the thread
    // is block - however we cannot do that here so instead, there
    // is a sacrificial "waker" thread who's only job is to sleep/wake and
    // signal to Nodejs that futures need to be run.
    //
    // The waker thread notifies the main thread of pending work by
    // running the futures executor within a threadsafe function
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
}
