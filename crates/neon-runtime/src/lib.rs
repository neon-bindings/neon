#[cfg(all(not(feature = "neon-sys"), not(feature = "napi")))]
compile_error!("The Neon runtime must have at least one of the `neon-sys` or `napi` backends enabled.");

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "napi")] {
        pub mod napi;
    }
}

cfg_if! {
    if #[cfg(feature = "neon-sys")] {
        pub mod nan;
        // The legacy variant is the default API as long as it's present.
        pub use crate::nan::*;
    } else if #[cfg(feature = "napi")] {
        // The N-API variant is only the default API if the legacy variant is disabled.
        pub use crate::napi::*;
    }
}
