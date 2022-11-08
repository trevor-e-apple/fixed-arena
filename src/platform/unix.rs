use libc;

use crate::platform::{FunctionsTrait, Platform};

impl FunctionsTrait for Platform {
    fn get_page_size() -> usize {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    }

    unsafe fn reserve(reserved: usize) -> *mut u8 {
        0 as *mut u8
    }

    unsafe fn release(base: *mut u8) {}

    unsafe fn commit(base: *mut u8, size: usize) {}

    unsafe fn decommit(base: *mut u8, free_from: usize, free_to: usize) {}
}
