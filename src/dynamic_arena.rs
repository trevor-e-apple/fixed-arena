use core::cell::Cell;

pub struct DynamicArena {
    base: *mut u8,
    used: Cell<usize>,
    pages_committed: Cell<usize>,
    pages_reserved: usize,
    page_size: usize,
}

#[cfg(target_os = "windows")]
mod windows_dynamic_arena;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_structs::TestStruct, platform::get_page_size};

    // TODO: document me
    #[test]
    fn test_with_capacity_reserve() {
        let page_size = get_page_size();
        DynamicArena::with_capacity_reserve(page_size, page_size);
    }

    #[test]
    fn test_alloc() {
        let page_size = get_page_size();
        let arena = DynamicArena::with_capacity_reserve(page_size, page_size);
        {
            let test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
            assert!(test.x == 0.0);
            assert!(test.y == 0.0);
        }
    }

    #[test]
    fn test_reset() {
        todo!();
    }

    #[test]
    fn test_alloc_over_committed() {
        unimplemented!();
    }

    #[test]
    fn test_alloc_to_capacity() {
        todo!();
    }

    #[test]
    fn test_alloc_no_reserves() {
        todo!();
    }

    /// A test for initializing an arena where we requested < 1 page worth of
    /// memory. The user should be able to use all the way up to one page
    #[test]
    fn test_alloc_under_page_size() {
        todo!();
    }

    /// A test for initializing an arena where we requested smaller reserves
    /// than the initial capacity
    #[test]
    fn test_arena_small_reserves() {
        todo!();
    }

    /// A test for initializing an arena where the reserves are not a multiple
    /// of the page size
    #[test]
    fn test_arena_fractional_page() {
        todo!();
    }
}
