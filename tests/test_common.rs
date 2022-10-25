// THIS CODE SHOULD BE AN EXACT COPY OF src/test_common.rs
use tea_arena::platform::get_page_size;

pub fn get_element_count() -> usize {
    4 * get_page_size()
}