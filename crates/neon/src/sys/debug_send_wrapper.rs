//! Wrapper that ensures types are always used from the same thread
//! in debug builds. It is a zero-cost in release builds.

pub(super) use wrapper::DebugSendWrapper;

#[cfg(debug_assertions)]
mod wrapper {
    use std::ops::Deref;

    #[repr(transparent)]
    pub struct DebugSendWrapper<T>(send_wrapper::SendWrapper<T>);

    impl<T> DebugSendWrapper<T> {
        pub fn new(value: T) -> Self {
            Self(send_wrapper::SendWrapper::new(value))
        }

        pub fn take(self) -> T {
            self.0.take()
        }
    }

    impl<T> Deref for DebugSendWrapper<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[cfg(not(debug_assertions))]
mod wrapper {
    use std::ops::Deref;

    #[repr(transparent)]
    pub struct DebugSendWrapper<T>(T);

    impl<T> DebugSendWrapper<T> {
        pub fn new(value: T) -> Self {
            Self(value)
        }

        pub fn take(self) -> T {
            self.0
        }
    }

    impl<T> Deref for DebugSendWrapper<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
