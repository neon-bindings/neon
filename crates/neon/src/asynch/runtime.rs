use std::cell::RefCell;
use std::future::Future;

use crate::context::internal::Env;
use futures::task::LocalSpawnExt;
use futures::executor::LocalSpawner;
use futures::executor::LocalPool;
use once_cell::unsync::Lazy;

use super::libuv::get_lib_uv;

thread_local! {
    static LOCAL_POOL: Lazy<RefCell<LocalPool>> = Lazy::new(|| RefCell::new(LocalPool::new()));
    static SPAWNER: Lazy<LocalSpawner> = Lazy::new(|| LOCAL_POOL.with(|ex| ex.borrow().spawner()) );
    static TASK_COUNT: Lazy<RefCell<usize>> = Lazy::new(|| Default::default() );
}

pub fn spawn_async_local(env: &Env, future: impl Future<Output = ()> + 'static) {
    SPAWNER.with(|ls| {
        ls.spawn_local(async {
            future.await;
            task_count_dec();
        })
        .unwrap();
    });

    // Delegate non-blocking polling of futures to libuv
    if task_count_inc() != 0 {
        return;
    }

    // Idle handle refers to a libuv task that runs while "idling".
    // This is not an idle state, rather an analogy to a car engine
    let uv = get_lib_uv(env);
    let mut task = uv.idle().unwrap();

    // The idle task will conduct a non-blocking poll of all local futures
    // and continue on pending futures allowing the poll to be non-blocking.
    // This repeats until no more futures are pending in the local set.
    task.start(move |mut task: libuv::IdleHandle| {
        if task_count() != 0 {
            LOCAL_POOL.with(|lp| lp.borrow_mut().run_until_stalled());
        } else {
            task.stop().unwrap();
        }
    })
    .unwrap();
}

fn task_count() -> usize {
    TASK_COUNT.with(|c| *c.borrow_mut())
}

fn task_count_inc() -> usize {
    let current = task_count();
    TASK_COUNT.with(|c| *c.borrow_mut() += 1);
    current
}

fn task_count_dec() -> usize {
    let current = task_count();
    TASK_COUNT.with(|c| *c.borrow_mut() -= 1);
    current
}
