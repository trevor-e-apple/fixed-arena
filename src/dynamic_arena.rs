use core::cell::Cell;

pub struct DynamicArena {
    base: *mut u8,
    used: Cell<usize>,
    pages_committed: usize,
    pages_reserved: usize,
    page_size: u32,
}

#[cfg(target_os = "windows")]
mod windows_dynamic_arena;
