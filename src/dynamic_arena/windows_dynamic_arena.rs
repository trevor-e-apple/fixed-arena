use core::cell::Cell;
use std::{alloc::Layout, ffi::c_void, ptr};

use windows::Win32::System::{
    Memory::{
        VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
        PAGE_READWRITE,
    }
};

use crate::{
    dynamic_arena::DynamicArena, errors::AllocError, platform::get_page_size,
};

impl DynamicArena {
    // TODO: more documentation with details on page sizes, and difference between
    // -- capacity and reserved
    /// reserved must be greater than or equal to capacity
    pub fn with_capacity_reserve(
        capacity: usize,
        reserved: usize,
    ) -> DynamicArena {
        assert!(capacity <= reserved);

        let page_size = get_page_size();

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
            pages_reserved,
            pages_committed: Cell::new(pages_committed),
            used: Cell::new(0),
        }
    }

    // TODO: inline
    // TODO: can this be moved out to code common to all implementations?
    /// converts page count to equivalent number of bytes
    fn page_to_size(&self, page_count: usize) -> usize {
        page_count * self.page_size
    }

    // TODO: inline
    // TODO: can this be moved out to code common to all implementations?
    /// gives the size of all reserved memory in bytes
    pub fn get_reserved(&self) -> usize {
        self.page_to_size(self.pages_reserved)
    }

    // TODO: inline
    // TODO: can this be moved out to code common to all implementations?
    /// gives the size of all committed memory in bytes
    pub fn get_committed(&self) -> usize {
        self.page_to_size(self.pages_committed.get())
    }

    // TODO: inline
    /// increases committed page count if there is not enough memory
    fn grow(&self, new_mem_needed: usize) -> Result<(), AllocError> {
        let mem_needed = self.used.get() + new_mem_needed;

        if mem_needed > self.get_reserved() {
            return Err(AllocError::AtCapacity);
        }

        let mem_committed = self.page_to_size(self.pages_committed.get());
        if mem_needed > mem_committed {
            // + 1 b/c we always need at least one page
            let pages_needed = (mem_needed / self.page_size) + 1;
            let double_pages_needed = 2 * pages_needed;
            let new_commit_page_count =
                if double_pages_needed < self.pages_reserved {
                    double_pages_needed
                } else {
                    self.pages_reserved
                };

            /*
            From microsoft docs on VirtualAlloc:
            "VirtualAlloc...can commit a page that is already committed.
            This means you can commit a range of pages, regardless of whether
            they have already been committed, and the function will not fail."
            */
            unsafe {
                VirtualAlloc(
                    Some(self.base as *const c_void),
                    self.page_to_size(new_commit_page_count),
                    MEM_COMMIT,
                    PAGE_READWRITE,
                );
                // TODO: error handling
            }

            self.pages_committed.set(new_commit_page_count);

            Ok(())
        } else {
            Ok(())
        }
    }

    // TODO: document me
    pub fn alloc<T>(&self, val: T) -> Result<&mut T, AllocError> {
        let layout = Layout::new::<T>();
        self.grow(layout.size())?;

        self.used.set(self.used.get() + layout.size());
        unsafe {
            let result = self.base.offset(layout.size() as isize) as *mut T;
            ptr::write(result, val);
            Ok(&mut *result)
        }
    }

    // TODO: reset with minimum size
    // TODO: just shrink without a reset
}

impl Drop for DynamicArena {
    fn drop(&mut self) {
        unsafe {
            VirtualFree(
                self.base as *mut c_void,
                self.pages_reserved * self.page_size,
                MEM_RELEASE,
            );
        }
    }
}
