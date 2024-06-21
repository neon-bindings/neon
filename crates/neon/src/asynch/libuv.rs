use std::mem::MaybeUninit;
use std::rc::Rc;

use crate::context::internal::Env;
use crate::sys::bindings::get_uv_event_loop;
use libuv::sys::uv_loop_t;
use libuv::Loop;
use once_cell::unsync::OnceCell;

thread_local! {
    pub static LIB_UV: OnceCell<Rc<Loop>> = OnceCell::new();
}

/// Gets a reference to Libuv
pub fn get_lib_uv<'a>(env: &Env) -> Rc<Loop> {
    LIB_UV.with(move |cell| {
        cell.get_or_init(move || {
            let mut result = MaybeUninit::uninit();
            unsafe { get_uv_event_loop(env.to_raw(), result.as_mut_ptr()) };
            let ptr = unsafe { *result.as_mut_ptr() };
            let ptr = ptr as *mut uv_loop_t;
            Rc::new(unsafe { libuv::r#loop::Loop::from_external(ptr) })
        })
        .clone()
    })
}
