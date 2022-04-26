use std::mem::MaybeUninit;

pub(crate) use bindings::*;

pub mod array;
pub mod arraybuffer;
pub mod async_work;
pub mod buffer;
pub mod call;
pub mod convert;
pub mod error;
pub mod external;
pub mod fun;
pub mod mem;
pub mod no_panic;
pub mod object;
pub mod primitive;
pub mod promise;
pub mod raw;
pub mod reference;
pub mod scope;
pub mod string;
pub mod tag;
pub mod typedarray;

mod bindings;

#[cfg(feature = "napi-4")]
pub mod tsfn;

#[cfg(feature = "napi-5")]
pub mod date;

#[cfg(feature = "napi-6")]
pub mod lifecycle;

/// Create a JavaScript `String`, panicking if unsuccessful
///
/// # Safety
/// * `env` is a `napi_env` valid for the current thread
/// * The returned value does not outlive `env`
unsafe fn string(env: Env, s: impl AsRef<str>) -> raw::Local {
    let s = s.as_ref();
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        create_string_utf8(
            env,
            s.as_bytes().as_ptr() as *const _,
            s.len(),
            result.as_mut_ptr(),
        ),
        Status::Ok,
    );

    result.assume_init()
}
