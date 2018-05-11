use raw::Local;
use std::mem;
use std::os::raw::c_int;
use std::os::raw::c_void;

/// An async callback. Uses a V8 persistent handle.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AsyncCallback {
    pub handle: *mut c_void,
}

impl AsyncCallback {
    pub fn new(callback: Local) -> AsyncCallback {
        unsafe {
            AsyncCallback {
                handle: new(callback),
            }
        }
    }

    pub fn call(&self, mut args: Vec<Local>) {
        unsafe {
            call(
                mem::transmute(self.handle),
                args.len() as i32,
                args.as_mut_ptr(),
            )
        }
    }

    // Implementing both `Copy` and `Drop` is unsound; however:
    // - The emit closure in `concurrent.rs` is bound to `Fn`, which requires `Copy`
    // - We still need to deallocate the persistent handle or we have a leak
    //
    // Instead of implementing `Drop`, we manually call this method.
    // Since we can manually guarantee that it will only be called once, we avoid
    // requiring our destructor to be idempotent.
    // 
    // See: https://github.com/rust-lang/rust/issues/20126
    // error[E0184]: the trait `Copy` may not be implemented for this type; the type has a destructor
    pub fn destroy(&mut self) {
        unsafe { delete(self.handle) }
    }
}

unsafe impl Send for AsyncCallback {}

extern "C" {
    #[link_name = "Neon_Async_Callback_New"]
    pub fn new(value: Local) -> *mut c_void;

    #[link_name = "Neon_Async_Callback_Call"]
    pub fn call(callback: *mut c_void, argc: c_int, argv: *mut Local);

    #[link_name = "Neon_Async_Callback_Delete"]
    pub fn delete(callback: *mut c_void);
}
