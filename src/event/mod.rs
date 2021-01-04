#[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
mod event_queue;

#[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
pub use self::event_queue::{EventQueue, EventQueueError};

#[cfg(feature = "event-handler-api")]
mod event_handler;

#[cfg(feature = "event-handler-api")]
pub use self::event_handler::EventHandler;
