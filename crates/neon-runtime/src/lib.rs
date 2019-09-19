extern crate cfg_if;

#[cfg(feature = "neon-sys")]
extern crate neon_sys;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "neon-sys")] {
        mod nan;
        pub use nan::*;
    } else {
        mod napi;
        pub use napi::*;
    }
}
