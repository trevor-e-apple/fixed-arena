#[cfg(target_os = "windows")]
mod windows_platform;

pub use windows_platform::get_page_size;