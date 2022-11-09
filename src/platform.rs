#[cfg(target_family = "windows")]
mod windows;

#[cfg(target_family = "unix")]
mod unix;

pub struct Platform;

pub trait PlatformFunctions {
    // TODO: document me
    fn get_page_size() -> usize;

    // TODO: finish documenting me
    // TODO: handle errors from system
    /// Reserves a section of virtual memory on the system
    unsafe fn reserve(reserved: usize) -> *mut u8;

    // TODO: finish documenting me
    // TODO: handle errors from system
    /// Releases the reservation that the application has for a section of
    /// virtual memory on the system
    unsafe fn release(base: *mut u8, size: usize);

    // TODO: finish documenting me
    // TODO: handle errors from system
    /// Commits a portion of virtual memory, starting from base and ending at
    /// base + size
    /// If a page in the range is already committed, it remains committed
    /// without issue
    unsafe fn commit(base: *mut u8, size: usize);

    // TODO: finish documenting me
    // TODO: handle errors from the system
    unsafe fn decommit(base: *mut u8, free_from: usize, free_size: usize);
}
