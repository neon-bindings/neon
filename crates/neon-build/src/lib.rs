extern crate cfg_if;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(windows)] {
        mod windows;

        pub use windows::*;
    } else if #[cfg(target_os = "macos")] {
        mod macos;

        pub use macos::*;
    } else {
        mod linux;

        pub use linux::*;
    }
}
