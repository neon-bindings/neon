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

#[cfg(feature = "tokio-rt-multi-thread")]
pub(crate) fn init(cx: &mut crate::context::ModuleContext) -> crate::result::NeonResult<()> {
    use once_cell::sync::OnceCell;
    use tokio::runtime::{Builder, Runtime};

    use crate::context::Context;

    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    super::RUNTIME.get_or_try_init(cx, |cx| {
        let runtime = RUNTIME
            .get_or_try_init(|| {
                #[cfg(feature = "tokio-rt-multi-thread")]
                let mut builder = Builder::new_multi_thread();

                #[cfg(not(feature = "tokio-rt-multi-thread"))]
                let mut builder = Builder::new_current_thread();

                builder.enable_all().build()
            })
            .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(Box::new(runtime))
    })?;

    Ok(())
}
