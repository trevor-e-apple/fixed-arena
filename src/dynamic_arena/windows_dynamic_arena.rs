use core::cell::Cell;
use std::{ffi::c_void, ptr::addr_of_mut};

use windows::Win32::System::{
    Memory::{
        VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
        PAGE_READWRITE,
    },
    SystemInformation::{GetSystemInfo, SYSTEM_INFO},
};

use crate::dynamic_arena::DynamicArena;

impl DynamicArena {
    // TODO: more documentation with details on page sizes, and difference between
    // -- capacity and reserved
    /// reserved must be greater than or equal to capacity
    pub fn with_capacity_reserve(
        capacity: usize,
        reserved: usize,
    ) -> DynamicArena {
        assert!(capacity < reserved);

        let page_size: u32;
        unsafe {
            let mut system_info = SYSTEM_INFO {
                ..Default::default()
            };
            GetSystemInfo(addr_of_mut!(system_info));
            page_size = system_info.dwPageSize;
        }

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

            // TODO: error handling
        }

        let pages_reserved =
            ((reserved as f64) / (page_size as f64)).ceil() as usize;
        let pages_committed =
            ((capacity as f64) / (page_size as f64)).ceil() as usize;

        DynamicArena {
            base,
            page_size,
            pages_committed,
            pages_reserved,
            used: Cell::new(0),
        }
    }
}

impl Drop for DynamicArena {
    fn drop(&mut self) {
        unsafe {
            VirtualFree(
                self.base as *mut c_void,
                self.pages_reserved * (self.page_size as usize),
                MEM_RELEASE,
            );
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
