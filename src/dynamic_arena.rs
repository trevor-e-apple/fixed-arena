use core::cell::Cell;

pub struct DynamicArena {
    base: *mut u8,
    used: Cell<usize>,
    committed: Cell<usize>,
    reserved: usize,
    page_size: usize,
}

#[cfg(target_os = "windows")]
mod windows_dynamic_arena;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        errors::AllocError, platform::get_page_size, test_structs::TestStruct,
    };

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
    fn test_alloc_zeroed() {
        let page_size = get_page_size();
        let arena = DynamicArena::with_capacity_reserve(page_size, page_size);
        {
            let test = arena.alloc_zeroed::<TestStruct>().unwrap();
            assert!(test.x == 0.0);
            assert!(test.y == 0.0);
        }
    }

    // Allocate until we have used all of the reserved space
    fn fill_arena(arena: &mut DynamicArena) {
        loop {
            match arena.alloc(TestStruct { x: 0.0, y: 0.0 }) {
                Ok(test) => {
                    assert!(test.x == 0.0);
                    assert!(test.y == 0.0);
                }
                Err(err) => {
                    if err == AllocError::AtCapacity {
                        break;
                    } else {
                        assert!(false);
                    }
                }
            }
        }
    }

    #[test]
    fn test_reset() {
        let page_size = get_page_size();
        let mut arena =
            DynamicArena::with_capacity_reserve(page_size, page_size);

        fill_arena(&mut arena);

        arena.reset();
        assert_eq!(arena.used.get(), 0);
        assert_eq!(arena.committed.get(), page_size);

        {
            let test = arena.alloc(TestStruct { x: 1.0, y: 2.0 }).unwrap();
            assert!(test.x == 1.0);
            assert!(test.y == 2.0);
        }
    }

    #[test]
    fn test_reset_without_shrink() {
        let page_size = get_page_size();
        let mut arena =
            DynamicArena::with_capacity_reserve(page_size, 2 * page_size);

        fill_arena(&mut arena);

        arena.reset();
        assert_eq!(arena.used.get(), 0);
        assert_eq!(arena.committed.get(), arena.reserved);

        {
            let test = arena.alloc(TestStruct { x: 1.0, y: 2.0 }).unwrap();
            assert!(test.x == 1.0);
            assert!(test.y == 2.0);
        }
    }

    #[test]
    fn test_reset_and_shrink() {
        let page_size = get_page_size();
        let mut arena =
            DynamicArena::with_capacity_reserve(page_size, 2 * page_size);

        fill_arena(&mut arena);

        arena.reset_and_shrink(page_size);
        assert_eq!(arena.used.get(), 0);
        assert_eq!(arena.committed.get(), page_size);

        // test that we can alloc and write after reset
        {
            let test = arena.alloc(TestStruct { x: 1.0, y: 2.0 }).unwrap();
            assert_eq!(test.x, 1.0);
            assert_eq!(test.y, 2.0);
        }
    }

    #[test]
    fn test_reset_and_shrink_page_offset() {
        let page_size = get_page_size();
        let mut arena =
            DynamicArena::with_capacity_reserve(page_size, 4 * page_size);

        fill_arena(&mut arena);

        // shrink to a size that is not a multiple of page size
        // should round up to the next page
        arena.reset_and_shrink((1.5 * page_size as f64) as usize);
        assert_eq!(arena.used.get(), 0);
        assert_eq!(arena.committed.get(), 2 * page_size);

        // test that we can alloc and write after reset
        {
            let test = arena.alloc(TestStruct { x: 1.0, y: 2.0 }).unwrap();
            assert!(test.x == 1.0);
            assert!(test.y == 2.0);
        }
    }

    // test allocating memory that is not a factor of the page size
    #[test]
    fn test_alloc_struct_not_page_size_factor() {
        unimplemented!();
    }

    #[test]
    fn test_shrink_to_zero() {
        unimplemented!();
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
