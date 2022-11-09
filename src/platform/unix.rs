use libc;
use std::ffi::c_void;

use crate::platform::{PlatformFunctions, Platform};

impl PlatformFunctions for Platform {
    fn get_page_size() -> usize {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    }

    unsafe fn reserve(reserved: usize) -> *mut u8 {
        let mmap_result = libc::mmap(
            0 as *mut c_void,
            reserved,
            libc::PROT_NONE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            0,
            0,
        );
        mmap_result as *mut u8
    }

    unsafe fn release(base: *mut u8, size: usize) {
        libc::munmap(base as *mut c_void, size);
    }

    unsafe fn commit(base: *mut u8, size: usize) {
        libc::mprotect(
            base as *mut c_void,
            size,
            libc::PROT_READ | libc::PROT_WRITE,
        );
    }

    unsafe fn decommit(base: *mut u8, free_from: usize, free_size: usize) {
        libc::mprotect(
            base.offset(free_from as isize) as *mut c_void,
            free_size,
            libc::PROT_NONE,
        );
    }
}
