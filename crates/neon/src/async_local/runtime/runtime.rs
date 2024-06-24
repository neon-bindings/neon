use std::cell::RefCell;
use std::future::Future;

use super::super::executor::LocalPool;
use super::super::executor::LocalSpawner;
use super::super::executor::ThreadNotifyRef;
use futures::task::LocalSpawnExt;
use futures::task::SpawnError;
use once_cell::unsync::Lazy;

thread_local! {
    static LOCAL_POOL: Lazy<RefCell<LocalPool>> = Lazy::default();
    static SPAWNER: Lazy<LocalSpawner> = Lazy::new(|| LOCAL_POOL.with(|ex| ex.borrow().spawner()));
}

pub struct LocalRuntime;

impl LocalRuntime {
    pub fn spawn_local(future: impl Future<Output = ()> + 'static) -> Result<(), SpawnError> {
        SPAWNER.with(move |ls| ls.spawn_local(future))
    }

    pub fn run_until_stalled(thread_notify: ThreadNotifyRef) {
        LOCAL_POOL.with(move |lp| lp.borrow_mut().run_until_stalled(thread_notify));
    }
}

