use std::ptr::addr_of_mut;
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

pub fn get_page_size() -> usize {
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
