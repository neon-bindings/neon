extern crate cfg_if;

#[cfg(all(not(feature = "neon-sys"), not(feature = "nodejs-sys")))]
compile_error!("The Neon runtime must have at least one of the `neon-sys` or `nodejs-sys` backends enabled.");

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "nodejs-sys")] {
        pub extern crate nodejs_sys;
        pub mod napi;
    }
}

cfg_if! {
    if #[cfg(feature = "neon-sys")] {
        extern crate neon_sys;
        pub mod nan;
        // The legacy variant is the default API as long as it's present.
        pub use nan::*;
    } else if #[cfg(feature = "nodejs-sys")] {
        // The N-API variant is only the default API if the legacy variant is disabled.
        pub use napi::*;
    }
}
