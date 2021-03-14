#[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
mod event_queue;

#[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
pub use self::event_queue::{EventQueue, EventQueueError};

#[cfg(all(not(feature = "napi-1"), feature = "event-handler-api"))]
mod event_handler;

#[cfg(all(not(feature = "napi-1"), feature = "event-handler-api"))]
pub use self::event_handler::EventHandler;

#[cfg(all(feature = "napi-1", feature = "event-handler-api"))]
compile_error!(
    "The `EventHandler` API is not supported with the N-API \
    backend. Use `EventQueue` instead."
);
