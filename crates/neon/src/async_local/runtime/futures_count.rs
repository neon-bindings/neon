use once_cell::unsync::Lazy;
use std::cell::RefCell;

thread_local! {
    static FUTURES_COUNT: Lazy<RefCell<usize>> = Lazy::default();
}

/// Local counter of the futures running on the current thread
///
/// By default the futures executor will keep the Nodejs process open.
/// This is desirable to avoid Nodejs from exiting before the futures
/// complete their execution.
///
/// For this reason, we need to keep a local counter of how many futures
/// are currently in flight so we can unref the executor when all of
/// the promises are settled.
pub struct LocalFuturesCount;

impl LocalFuturesCount {
    pub fn count() -> usize {
        FUTURES_COUNT.with(|c| *c.borrow_mut())
    }

    pub fn increment() -> usize {
        let futures_count = LocalFuturesCount::count();
        FUTURES_COUNT.with(|c| *c.borrow_mut() += 1);
        futures_count
    }

    pub fn decrement() -> usize {
        let futures_count = LocalFuturesCount::count();
        FUTURES_COUNT.with(|c| *c.borrow_mut() -= 1);
        futures_count
    }
}
