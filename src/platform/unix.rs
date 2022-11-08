// TODO: document me
pub fn get_page_size() -> usize {
    0
}

// TODO: finish documenting me
// TODO: handle errors from system
/// Reserves a section of virtual memory on the system
pub unsafe fn reserve(reserved: usize) -> *mut u8 {
    0 as *mut u8
}

// TODO: finish documenting me
/// Releases the reservation that the application has for a section of virtual
/// memory on the system
pub unsafe fn release(base: *mut u8) {}

// TODO: finish documenting me
// TODO: handle errors from system
/// Commits a portion of virtual memory, starting from base and ending at
/// base + size
/// If a page in the range is already committed, it remains committed without
/// issue
pub unsafe fn commit(base: *mut u8, size: usize) {}

// TODO: finish documenting me
pub unsafe fn decommit(base: *mut u8, free_from: usize, free_to: usize) {}
