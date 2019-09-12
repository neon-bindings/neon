extern crate cfg_if;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature =  "n-api")] {
        mod napi;
        pub use napi::*;
    } else {
        mod nan;
        pub use nan::*;
    }
}
