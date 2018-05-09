use std::mem;
use std::ops::Drop;
use uv::{uv_async_init, uv_async_send, uv_async_t, uv_close, uv_default_loop, uv_handle_t};

extern "C" fn on_wake(handle: *mut uv_async_t) {
    unsafe {
        let handle = &mut *handle;
        let closure: &mut Box<FnMut()> = mem::transmute(handle.data);
        *&closure()
    }
}
unsafe extern "C" fn on_handle_close(handle: *mut uv_handle_t) {
    // Prevent the handle from leaking since we used Box::into_raw earlier
    let _ = Box::from_raw(handle);
}

// A safe Rust wrapper around libuv's uv_async_t.
#[derive(Debug)]
pub struct AsyncHandle {
    internal_handle: *mut uv_async_t,
}

impl AsyncHandle {
    pub fn new() -> AsyncHandle {
        let internal_handle = uv_async_t {
            ..Default::default()
        };
        let internal_handle = Box::new(internal_handle);
        let internal_handle = Box::into_raw(internal_handle) as *mut uv_async_t;
        unsafe {
            uv_async_init(uv_default_loop(), internal_handle, Some(on_wake));
        }

        AsyncHandle { internal_handle }
    }

    pub fn send(&self) {
        unsafe {
            uv_async_send(self.internal_handle);
        }
    }

    pub fn close(&self) {
        unsafe {
            let handle = &mut *self.internal_handle;
            uv_close(mem::transmute(handle), Some(on_handle_close));
        }
    }

    pub fn wake_event_loop<F>(&self, callback: F)
    where
        F: FnMut(),
    {
        let cb: Box<Box<FnMut()>> = Box::new(Box::new(callback));
        unsafe {
            let handle = &mut *self.internal_handle;
            handle.data = mem::transmute(cb);
            uv_async_send(self.internal_handle);
        }
    }
}

impl Drop for AsyncHandle {
    fn drop(&mut self) {
        self.close();
    }
}

unsafe impl Send for AsyncHandle {}
unsafe impl Sync for AsyncHandle {}
