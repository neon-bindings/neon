use std::{future::Future, pin::Pin};

use crate::{context::Cx, thread::LocalKey};

#[cfg(feature = "tokio-rt")]
pub(crate) mod tokio;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub(crate) static RUNTIME: LocalKey<Box<dyn Runtime>> = LocalKey::new();

pub trait Runtime: Send + Sync + 'static {
    fn spawn(&self, fut: BoxFuture);
}

/// Register a [`Future`] executor runtime globally to the addon.
///
/// Returns `Ok(())` if a global executor has not been set and `Err(runtime)` if it has.
///
/// If the `tokio` feature flag is enabled and the addon does not provide a
/// [`#[neon::main]`](crate::main) function, a multithreaded tokio runtime will be
/// automatically registered.
///
/// **Note**: Each instance of the addon will have its own runtime. It is recommended
/// to initialize the async runtime once in a process global and share it across instances.
///
/// ```
/// # #[cfg(feature = "tokio-rt-multi-thread")]
/// # fn example() {
/// # use neon::prelude::*;
/// use once_cell::sync::OnceCell;
/// use tokio::runtime::Runtime;
///
/// static RUNTIME: OnceCell<Runtime> = OnceCell::new();
///
/// #[neon::main]
/// fn main(mut cx: ModuleContext) -> NeonResult<()> {
///     let runtime = RUNTIME
///         .get_or_try_init(Runtime::new)
///         .or_else(|err| cx.throw_error(err.to_string()))?;
///
///     let _ = neon::set_global_executor(&mut cx, runtime);
///
///     Ok(())
/// }
/// # }
/// ```
pub fn set_global_executor<R>(cx: &mut Cx, runtime: R) -> Result<(), R>
where
    R: Runtime,
{
    if RUNTIME.get(cx).is_some() {
        return Err(runtime);
    }

    RUNTIME.get_or_init(cx, || Box::new(runtime));

    Ok(())
}
