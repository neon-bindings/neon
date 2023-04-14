//! Raw bindings to [Node-API][node-api]
//!
//! [Node-API][node-api] is Node.js's API for building native addons. Neon is
//! predominantly a safe wrapper for Node-API and most users should prefer the
//! the high level abstractions [outside](crate) of the sys module.
//!
//! However, directly using Node-API can be a useful tool for accessing low
//! level functionality not exposed by Neon or experimenting with extensions
//! to Neon without needing to fork the project.
//!
//! [node-api]: https://nodejs.org/api/n-api.html
//!
//! ## Initialization
//!
//! Before any Node-API functions may be used, [`setup`](setup) must be called at
//! least once.
//!
//! ```rust,no_run
//! # #[cfg(feature = "sys")]
//! # {
//! # let env = std::ptr::null_mut().cast();
//! unsafe { neon::sys::setup(env); }
//! # }
//! ```
//! **Note**: It is unnecessary to call [`setup`](setup) if
//! [`#[neon::main]`](crate::main) is used to initialize the addon.
//!
//! ## Safety
//!
//! The following are guidelines for ensuring safe usage of Node-API in Neon
//! but, do not represent a comprehensive set of safety rules. If possible,
//! users should avoid calling Neon methods or holding references to structures
//! created by Neon when calling Node-API functions directly.
//!
//! ### Env
//!
//! Neon ensures safety by carefully restricting access to [`Env`](bindings::Env)
//! by wrapping it in a [`Context`](crate::context::Context). Usages of `Env`
//! should follow Neon's borrowing rules of `Context`.
//!
//! It is unsound to use an `Env` if Rust's borrowing rules would prevent usage
//! of the in scope `Context`.
//!
//! ### Values
//!
//! Neon [value types](crate::types) encapsulate references to
//! [JavaScript values](bindings::Value) with a _known type_. It is unsound to
//! construct a Neon value with a [`Value`](bindings::Value) of the incorrect type.
//!
//! ## Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "sys")]
//! # {
//! # let env = std::ptr::null_mut().cast();
//! use neon::{context::SysContext, prelude::*, sys::bindings};
//!
//! let cx = unsafe {
//!     neon::sys::setup(env);
//!
//!     SysContext::from_raw(env)
//! };
//!
//! let raw_string: bindings::Value = cx.string("Hello, World!").to_raw();
//! let js_string = unsafe { JsString::from_raw(&cx, raw_string) };
//! # }
//! ```
use std::{mem::MaybeUninit, sync::Once};

// Bindings are re-exported here to minimize changes.
// Follow-up issue: https://github.com/neon-bindings/neon/issues/982
pub(crate) use bindings::*;

pub(crate) mod array;
pub(crate) mod arraybuffer;
pub(crate) mod async_work;
pub(crate) mod buffer;
pub(crate) mod call;
pub(crate) mod convert;
pub(crate) mod error;
pub(crate) mod external;
pub(crate) mod fun;
pub(crate) mod mem;
pub(crate) mod no_panic;
pub(crate) mod object;
pub(crate) mod primitive;
pub(crate) mod promise;
pub(crate) mod raw;
pub(crate) mod reference;
pub(crate) mod scope;
pub(crate) mod string;
pub(crate) mod tag;
pub(crate) mod typedarray;

pub mod bindings;

#[cfg(feature = "napi-4")]
pub(crate) mod tsfn;

#[cfg(feature = "napi-5")]
pub(crate) mod date;

mod debug_send_wrapper;
#[cfg(feature = "napi-6")]
pub(crate) mod lifecycle;

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

static SETUP: Once = Once::new();

/// Loads Node-API symbols from the host process.
///
/// Must be called at least once before using any functions in bindings or
/// they will panic.
///
/// # Safety
/// `env` must be a valid `napi_env` for the current thread
pub unsafe fn setup(env: Env) {
    SETUP.call_once(|| load(env).expect("Failed to load N-API symbols"));
}
