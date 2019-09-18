extern crate cfg_if;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature =  "n-api")] {
        pub extern crate nodejs_sys;
        mod napi;
        pub use napi::*;

        #[doc(hidden)]
        pub use nodejs_sys as sys;
    } else {
        mod nan;
        pub use nan::*;
    }
}
