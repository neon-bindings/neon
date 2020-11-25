#[cfg(all(not(windows), not(target_os = "macos")))]
mod linux;
#[cfg(all(not(windows), not(target_os = "macos")))]
pub(crate) use linux::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::*;
