use std::{ffi::c_void, ptr::addr_of_mut};

use windows::Win32::System::{
    Memory::{
        VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_DECOMMIT, MEM_RELEASE,
        MEM_RESERVE, PAGE_READWRITE,
    },
    SystemInformation::{GetSystemInfo, SYSTEM_INFO},
};

use crate::platform::{PlatformFunctions, Platform};

impl PlatformFunctions for Platform {
    fn get_page_size() -> usize {
        let page_size: u32;
        unsafe {
            let mut system_info = SYSTEM_INFO {
                ..Default::default()
            };
            GetSystemInfo(addr_of_mut!(system_info));
            page_size = system_info.dwPageSize;
        }

        page_size as usize
    }

    unsafe fn reserve(reserved: usize) -> *mut u8 {
        VirtualAlloc(
            None, // let the system allocate the region
            reserved,
            MEM_RESERVE,
            PAGE_READWRITE,
        ) as *mut u8
    }

    unsafe fn release(base: *mut u8, _size: usize) {
        VirtualFree(base as *mut c_void, 0, MEM_RELEASE);
    }

    // TODO: finish documenting me
    // TODO: handle errors from system
    /// Commits a portion of virtual memory, starting from base and ending at
    /// base + size
    /// If a page in the range is already committed, it remains committed without
    /// issue
    unsafe fn commit(base: *mut u8, size: usize) {
        /*
        From microsoft docs on VirtualAlloc:
        "VirtualAlloc...can commit a page that is already committed.
        This means you can commit a range of pages, regardless of whether
        they have already been committed, and the function will not fail."
        */
        VirtualAlloc(
            Some(base as *const c_void),
            size,
            MEM_COMMIT,
            PAGE_READWRITE,
        );
    }

    // TODO: finish documenting me
    unsafe fn decommit(base: *mut u8, free_from: usize, free_size: usize) {
        VirtualFree(
            base.offset(free_from as isize) as *mut c_void,
            free_size,
            MEM_DECOMMIT,
        );
    }
}
