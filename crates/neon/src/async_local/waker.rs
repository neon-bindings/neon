use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;

use crate::sys;

use super::executor::wait_for_wake;
use super::executor::ThreadNotify;
use super::executor::ThreadNotifyRef;
use once_cell::unsync::Lazy;

thread_local! {
    static WAKER_THREAD: Lazy<Sender<WakerEvent>> = Lazy::new(LocalWaker::start_waker_thread);
}

pub type WakerInit = sys::tsfn::ThreadsafeFunction<ThreadNotifyRef>;

pub enum WakerEvent {
    Init(WakerInit),
    Next,
    Done,
}

/// The futures waker that coordinates with the futures executor to notify
/// the main thread to pause and resume execution of futures.
///
/// The waker is implemented as a dedicated system thread which is parked
/// by the local futures executor while waiting for futures to resume work.
///
/// Once woken up, the waker resumes execution of futures on the JavaScript
/// thread by triggering a napi threadsafe function to poll the futures in
/// the local pool until no more progress can be made before yielding back
/// to the Nodejs event loop.
///
/// This allows for the execution of Rust futures to integrate with the
/// Nodejs event loop without blocking either
pub struct LocalWaker;

impl LocalWaker {
    pub fn send(event: WakerEvent) {
        WAKER_THREAD
            .with(|tx| tx.send(event))
            .expect("Unable to communicate with waker");
    }

    fn start_waker_thread() -> Sender<WakerEvent> {
        let (tx, rx) = channel();

        // Dedicated waker thread to use for waiting on pending futures
        thread::spawn(move || {
            let thread_notify = ThreadNotify::new();
            let mut handle = None::<WakerInit>;

            while let Ok(event) = rx.recv() {
                match event {
                    WakerEvent::Init(incoming) => {
                        if handle.replace(incoming).is_some() {
                            panic!("Handle already init");
                        };
                        let Some(ref handle) = handle else {
                            panic!("No handle");
                        };
                        handle.call(thread_notify.clone(), None).ok();
                    }
                    WakerEvent::Next => {
                        wait_for_wake(&thread_notify);
                        let Some(ref handle) = handle else {
                            panic!("No handle");
                        };
                        handle.call(thread_notify.clone(), None).ok();
                    }
                    WakerEvent::Done => {
                        if let Some(handle) = handle.take() {
                            drop(handle);
                        }
                    }
                };
            }
        });

        tx
    }
}
