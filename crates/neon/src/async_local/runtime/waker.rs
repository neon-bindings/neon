use std::sync::mpsc::channel;
use std::sync::mpsc::SendError;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use super::super::executor::wait_for_wake;
use super::super::executor::ThreadNotify;
use once_cell::unsync::Lazy;

thread_local! {
    static WAKER_THREAD: Lazy<Sender<WakerEvent>> = Lazy::new(LocalWaker::start_waker_thread);
}

pub type WakerInit = Arc<u32>;

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
pub  struct LocalWaker;

impl LocalWaker {
    pub fn send(event: WakerEvent) -> Result<(), SendError<WakerEvent>> {
        WAKER_THREAD.with(|tx| tx.send(event))
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
                            // Error
                        };
                        // Call JS
                    }
                    WakerEvent::Next => {
                        wait_for_wake(&thread_notify);
                        // Call JS
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

