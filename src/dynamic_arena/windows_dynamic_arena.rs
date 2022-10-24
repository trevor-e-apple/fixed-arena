use core::cell::Cell;
use std::{alloc::Layout, ffi::c_void, ptr, slice};

use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_DECOMMIT, MEM_RELEASE,
    MEM_RESERVE, PAGE_READWRITE,
};

use crate::{
    dynamic_arena::DynamicArena, errors::AllocError, platform::get_page_size,
};

impl DynamicArena {
    // TODO: more documentation with details on page sizes, and difference between
    // -- capacity and reserved
    /// reserved must be greater than or equal to capacity
    /// It is recommended that capacity and reserved are multiples of page size
    pub fn with_capacity_reserve(
        capacity: usize,
        reserved: usize,
    ) -> DynamicArena {
        assert!(capacity <= reserved);

        let base: *mut u8;
        unsafe {
            // reserve the pages
            base = VirtualAlloc(
                None, // let the system allocate the region
                reserved,
                MEM_RESERVE,
                PAGE_READWRITE,
            ) as *mut u8;

            // allocate the starting capacity
            VirtualAlloc(
                Some(base as *const c_void),
                capacity,
                MEM_COMMIT,
                PAGE_READWRITE,
            );

            // TODO: error handling
        }

        let page_size = get_page_size();
        DynamicArena {
            base,
            reserved,
            page_size,
            committed: Cell::new(capacity),
            used: Cell::new(0),
        }
    }

    // TODO: inline
    /// increases committed page count if there is not enough memory
    fn grow(&self, new_mem_needed: usize) -> Result<(), AllocError> {
        let mem_needed = self.used.get() + new_mem_needed;

        if mem_needed > self.reserved {
            return Err(AllocError::AtCapacity);
        } else if mem_needed > self.committed.get() {
            /*
            From microsoft docs on VirtualAlloc:
            "VirtualAlloc...can commit a page that is already committed.
            This means you can commit a range of pages, regardless of whether
            they have already been committed, and the function will not fail."
            */
            let double_mem_needed = 2 * mem_needed;
            let mem_to_commit = if double_mem_needed <= self.reserved {
                double_mem_needed
            } else {
                self.reserved
            };
            unsafe {
                VirtualAlloc(
                    Some(self.base as *const c_void),
                    mem_to_commit,
                    MEM_COMMIT,
                    PAGE_READWRITE,
                );
                // TODO: error handling
            }
            self.committed.set(mem_to_commit);

            Ok(())
        } else {
            Ok(())
        }
    }

    // TODO: document me
    fn get_alloc_ptr(&self, layout: Layout) -> Result<*mut u8, AllocError> {
        self.grow(layout.size())?;

        let result: *mut u8 =
            unsafe { self.base.offset(self.used.get() as isize) };
        self.used.set(self.used.get() + layout.size());
        Ok(result)
    }

    // TODO: document me
    pub fn alloc<T>(&self, val: T) -> Result<&mut T, AllocError> {
        let layout = Layout::new::<T>();
        let result_ptr = self.get_alloc_ptr(layout)?;

        unsafe {
            let result = result_ptr as *mut T;
            ptr::write(result, val);
            Ok(&mut *result)
        }
    }

    pub fn alloc_zeroed<T>(&self) -> Result<&mut T, AllocError> {
        let layout = Layout::new::<T>();
        let result_ptr = self.get_alloc_ptr(layout)?;

        unsafe {
            let result = result_ptr as *mut T;
            ptr::write_bytes(result, 0, 1);
            Ok(&mut *result)
        }
    }

    pub fn alloc_array<T>(
        &self,
        val: T,
        count: usize,
    ) -> Result<&mut [T], AllocError>
    where
        T: Clone,
    {
        let layout =
            Layout::array::<T>(count).expect("Bad count value for array");
        let result_ptr = self.get_alloc_ptr(layout)?;

        let result: &mut [T];
        unsafe {
            let pointer = result_ptr as *mut T;
            let isize_count = count as isize;
            for index in 0..isize_count {
                ptr::write(pointer.offset(index), val.clone());
            }
            result = slice::from_raw_parts_mut(pointer, count);
        }

        Ok(result)
    }

    pub fn alloc_zeroed_array<T>(
        &self,
        count: usize,
    ) -> Result<&mut [T], AllocError>
    where
        T: Clone,
    {
        let layout =
            Layout::array::<T>(count).expect("Bad count value for array");
        let result_ptr = self.get_alloc_ptr(layout)?;

        let result: &mut [T];
        unsafe {
            let pointer = result_ptr as *mut T;
            ptr::write_bytes(pointer, 0, count);
            result = slice::from_raw_parts_mut(pointer, count);
        }

        Ok(result)
    }

    pub fn alloc_uninitialized_array<T>(
        &self,
        count: usize,
    ) -> Result<&mut [T], AllocError>
    where
        T: Clone,
    {
        let layout =
            Layout::array::<T>(count).expect("Bad count value for array");
        let result_ptr = self.get_alloc_ptr(layout)?;

        let result: &mut [T];
        unsafe {
            let pointer = result_ptr as *mut T;
            result = slice::from_raw_parts_mut(pointer, count);
        }

        Ok(result)
    }

    // TODO: more documentation, examples
    /// Reset the arena. Set the used value to 0
    pub fn reset(&mut self) {
        self.used.set(0);
    }

    // TODO: more documentation, examples
    /// Reset the arena and then shrink the committed memory down to new_size
    /// it's recommended that new_size is a multiple of page size
    pub fn reset_and_shrink(&mut self, new_size: usize) {
        self.reset();

        // we only have to shrink if new_size is less than the committed size
        if new_size < self.committed.get() {
            let remainder = new_size % self.page_size;
            let free_from = if remainder == 0 {
                new_size
            } else {
                new_size + remainder
            };
            let free_to = self.reserved - free_from;
            unsafe {
                VirtualFree(
                    self.base.offset(free_from as isize) as *mut c_void,
                    free_to,
                    MEM_DECOMMIT,
                );
                // TODO: error handling
            }
            self.committed.set(free_from);
        }
    }
}

impl Drop for DynamicArena {
    fn drop(&mut self) {
        /*
        From microsoft docs on VirtualFree:
        If the dwFreeType parameter is MEM_RELEASE, this parameter must be 0
        (zero). The function frees the entire region that is reserved in the
        initial allocation call to VirtualAlloc.
        */
        unsafe {
            VirtualFree(self.base as *mut c_void, 0, MEM_RELEASE);
        }
    }
}
