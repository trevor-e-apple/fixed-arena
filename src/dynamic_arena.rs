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
    use std::mem::size_of;

    use super::*;
    use crate::{
        errors::AllocError,
        platform::get_page_size,
        test_structs::{I32Struct, TestStruct, ThreeByteStruct},
    };

    mod alloc_struct {
        use super::*;

        #[test]
        fn test_alloc() {
            let page_size = get_page_size();
            let arena =
                DynamicArena::with_capacity_reserve(page_size, page_size);
            {
                let test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
                assert!(test.x == 0.0);
                assert!(test.y == 0.0);
            }
        }

        #[test]
        fn test_alloc_zeroed() {
            let page_size = get_page_size();
            let arena =
                DynamicArena::with_capacity_reserve(page_size, page_size);
            {
                let test = arena.alloc_zeroed::<TestStruct>().unwrap();
                assert!(test.x == 0.0);
                assert!(test.y == 0.0);
            }
        }
    }

    mod grow_shrink {
        use super::*;

        // TODO: document me
        #[test]
        fn test_with_capacity_reserve() {
            let page_size = get_page_size();
            DynamicArena::with_capacity_reserve(page_size, page_size);
        }

        /// Allocate until we have used all of the reserved space
        fn fill_arena<T>(arena: &mut DynamicArena, val: T)
        where
            T: Copy,
        {
            loop {
                match arena.alloc(val) {
                    Ok(_) => {}
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

            fill_arena(
                &mut arena,
                TestStruct {
                    ..Default::default()
                },
            );

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

            fill_arena(
                &mut arena,
                TestStruct {
                    ..Default::default()
                },
            );

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

            fill_arena(
                &mut arena,
                TestStruct {
                    ..Default::default()
                },
            );

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

            fill_arena(
                &mut arena,
                TestStruct {
                    ..Default::default()
                },
            );

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

        #[test]
        fn test_shrink_to_zero() {
            let page_size = get_page_size();
            let mut arena =
                DynamicArena::with_capacity_reserve(page_size, 4 * page_size);

            fill_arena(
                &mut arena,
                TestStruct {
                    ..Default::default()
                },
            );

            // shrink to zero. nothing should be committed
            arena.reset_and_shrink(0);
            assert_eq!(arena.used.get(), 0);
            assert_eq!(arena.committed.get(), 0);

            // refill after shrink to zero
            fill_arena(
                &mut arena,
                TestStruct {
                    ..Default::default()
                },
            );
            assert_eq!(arena.committed.get(), arena.reserved);
        }

        /// test allocating memory that is a factor of the page size
        /// this also tests allocating with no reserves left over
        #[test]
        fn test_alloc_struct_page_size_factor() {
            let page_size = get_page_size();
            let mut arena =
                DynamicArena::with_capacity_reserve(page_size, 4 * page_size);

            fill_arena::<u8>(&mut arena, 0);
            assert_eq!(arena.used.get(), arena.committed.get());
            assert_eq!(arena.committed.get(), arena.reserved);
        }

        /// test allocating memory that is not a factor of the page size
        #[test]
        fn test_alloc_struct_not_page_size_factor() {
            let page_size = get_page_size();
            let mut arena =
                DynamicArena::with_capacity_reserve(page_size, 4 * page_size);

            fill_arena(
                &mut arena,
                ThreeByteStruct {
                    ..Default::default()
                },
            );
            assert!(arena.used.get() < arena.committed.get());
            // check that all of the memory that could have been used was used
            assert!(
                (arena.committed.get() - arena.used.get())
                    < size_of::<ThreeByteStruct>()
            );
            assert_eq!(arena.committed.get(), arena.reserved);
        }

        /// tests that the committed memory does not grow unnecessarily
        /// and that it grows exactly when it runs out of memory
        #[test]
        fn test_alloc_to_capacity() {
            let page_size = get_page_size();
            let arena =
                DynamicArena::with_capacity_reserve(page_size, 4 * page_size);

            for _ in 0..page_size {
                arena.alloc_zeroed::<u8>().unwrap();
            }
            assert_eq!(arena.committed.get(), page_size);

            arena.alloc_zeroed::<u8>().unwrap();
            assert!(arena.committed.get() > page_size);
        }

        /// A test for initializing an arena where we requested < 1 page worth of
        /// memory.
        #[test]
        fn test_alloc_under_page_size() {
            let page_size = get_page_size();
            let mut arena = DynamicArena::with_capacity_reserve(
                page_size / 2,
                page_size / 2,
            );

            fill_arena::<u8>(&mut arena, 0);
            assert_eq!(arena.committed.get(), page_size / 2);
            assert_eq!(arena.used.get(), page_size / 2);
        }

        /// A test for initializing an arena where we requested smaller reserves
        /// than the initial capacity
        #[test]
        #[should_panic]
        fn test_arena_small_reserves() {
            let page_size = get_page_size();
            let mut arena =
                DynamicArena::with_capacity_reserve(page_size, page_size / 2);
            arena.reset();
        }

        /// A test for initializing an arena where the reserves are not a multiple
        /// of the page size
        #[test]
        fn test_arena_fractional_page() {
            let page_size = get_page_size();
            let mut arena = DynamicArena::with_capacity_reserve(
                page_size,
                3 * page_size / 2,
            );

            fill_arena::<u8>(&mut arena, 0);
            assert_eq!(arena.committed.get(), arena.reserved);
            assert_eq!(arena.used.get(), arena.reserved);
        }
    }

    mod alloc_array {
        use super::*;

        #[test]
        fn test_alloc_array() {
            let page_size = get_page_size();
            let arena =
                DynamicArena::with_capacity_reserve(page_size, page_size);

            let init_value = TestStruct {
                ..Default::default()
            };
            let test = arena
                .alloc_array(
                    init_value,
                    arena.reserved / size_of::<TestStruct>(),
                )
                .unwrap();
            for element in test.iter() {
                assert_eq!(*element, init_value);
            }
            assert_eq!(arena.used.get(), arena.committed.get());
            assert_eq!(arena.committed.get(), arena.reserved);
        }

        #[test]
        fn test_alloc_array_non_default() {
            let page_size = get_page_size();
            let arena =
                DynamicArena::with_capacity_reserve(page_size, page_size);

            let init_value = TestStruct { x: 1.0, y: 2.0 };
            let test = arena
                .alloc_array(
                    init_value,
                    arena.reserved / size_of::<TestStruct>(),
                )
                .unwrap();
            for element in test.iter() {
                assert_eq!(*element, init_value);
            }
            assert_eq!(arena.used.get(), arena.committed.get());
            assert_eq!(arena.committed.get(), arena.reserved);
        }

        #[test]
        fn test_alloc_zeroed_array() {
            let page_size = get_page_size();
            let arena =
                DynamicArena::with_capacity_reserve(page_size, page_size);

            let test = arena
                .alloc_zeroed_array::<I32Struct>(
                    arena.reserved / size_of::<I32Struct>(),
                )
                .unwrap();
            for element in test.iter() {
                assert_eq!(element.x, 0);
                assert_eq!(element.y, 0);
            }
            assert_eq!(arena.used.get(), arena.committed.get());
            assert_eq!(arena.committed.get(), arena.reserved);
        }
    }
}
