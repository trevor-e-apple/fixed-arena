use core::cell::Cell;
use std::ffi::c_void;

use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
    PAGE_READWRITE,
};

use crate::dynamic_arena::DynamicArena;

impl DynamicArena {
    // TODO: more documentation with details on page sizes
    /// reserved must be greater than or equal to capacity
    pub fn with_capacity_reserve(
        capacity: usize, reserved: usize
    ) -> DynamicArena {
        assert!(capacity < reserved);

        let base: *mut u8;
        unsafe {
            // reserve the pages
            base = VirtualAlloc(
                None, // let the system allocate the region
                reserved,
                MEM_RESERVE,
                PAGE_READWRITE,
            ) as *mut u8;

            // allocate the starting amount
            VirtualAlloc(
                Some(base as *const c_void),
                capacity,
                MEM_COMMIT,
                PAGE_READWRITE,
            );
        }

        DynamicArena {
            base,
            used: Cell::new(0),
            capacity: capacity,
            reserved: reserved
        }
    }
}

impl Drop for DynamicArena {
    fn drop(&mut self) {
        unsafe {
            VirtualFree(self.base as *mut c_void, self.reserved, MEM_RELEASE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: document me
    #[test]
    fn test_with_capacity_reserve() {
        DynamicArena::with_capacity_reserve(1024, 2048);
    }
}
