// THIS CODE SHOULD BE AN EXACT COPY OF tests/test_common.rs
use crate::platform::get_page_size;

pub fn get_element_count() -> usize {
    4 * get_page_size()
}