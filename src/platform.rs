#[cfg(target_os = "windows")]
mod windows_platform;

#[cfg(target_os = "windows")]
pub use windows_platform::*;
