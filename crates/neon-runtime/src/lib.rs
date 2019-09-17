extern crate cfg_if;

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
