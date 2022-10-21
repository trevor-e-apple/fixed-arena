use core::cell::Cell;

pub struct DynamicArena {
    base: *mut u8,
    used: Cell<usize>,
    capacity: usize, // TODO: change this to pages_committed
    reserved: usize // TODO: change this to pages_reserved
    // TODO: track page size
}

#[cfg(target_os = "windows")]
mod windows_dynamic_arena;