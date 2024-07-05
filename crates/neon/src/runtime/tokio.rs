use std::sync::Arc;

use super::{BoxFuture, Runtime};

impl Runtime for tokio::runtime::Runtime {
    fn spawn(&self, fut: BoxFuture) {
        spawn(self.handle(), fut);
    }
}

impl Runtime for Arc<tokio::runtime::Runtime> {
    fn spawn(&self, fut: BoxFuture) {
        spawn(self.handle(), fut);
    }
}

impl Runtime for &'static tokio::runtime::Runtime {
    fn spawn(&self, fut: BoxFuture) {
        spawn(self.handle(), fut);
    }
}

impl Runtime for tokio::runtime::Handle {
    fn spawn(&self, fut: BoxFuture) {
        spawn(self, fut);
    }
}

impl Runtime for &'static tokio::runtime::Handle {
    fn spawn(&self, fut: BoxFuture) {
        spawn(self, fut);
    }
}

fn spawn(handle: &tokio::runtime::Handle, fut: BoxFuture) {
    #[allow(clippy::let_underscore_future)]
    let _ = handle.spawn(fut);
}
