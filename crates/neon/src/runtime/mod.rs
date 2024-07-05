use std::{future::Future, pin::Pin};

#[cfg(feature = "tokio-rt")]
mod tokio;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub trait Runtime: Send + Sync + 'static {
    fn spawn(&self, fut: BoxFuture);
}
