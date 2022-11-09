use core::cell::Cell;

pub struct DynamicArena {
    base: *mut u8,
    used: Cell<usize>,
    committed: Cell<usize>,
    reserved: usize,
    page_size: usize,
}

use std::{alloc::Layout, ptr, slice};

use crate::errors::AllocError;
use crate::platform::{PlatformFunctions, Platform};

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
            base = Platform::reserve(reserved);

            Platform::commit(base, capacity);

            // TODO: error handling
        }

        let page_size = Platform::get_page_size();
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
            let double_mem_needed = 2 * mem_needed;
            let mem_to_commit = if double_mem_needed <= self.reserved {
                double_mem_needed
            } else {
                self.reserved
            };
            unsafe {
                Platform::commit(self.base, mem_to_commit);
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
            let free_size = self.reserved - free_from;
            unsafe {
                Platform::decommit(self.base, free_from, free_size);
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
            Platform::release(self.base, self.reserved);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::platform::{PlatformFunctions, Platform};
    use crate::{
        errors::AllocError,
        test_common::{
            I32Struct, LargerStruct, MixedStruct, SmallStruct, SmallerStruct,
            TestStruct, ThreeByteStruct,
        },
    };

    mod alloc_struct {
        use super::*;

        #[test]
        fn alloc() {
            let page_size = Platform::get_page_size();
            let arena =
                DynamicArena::with_capacity_reserve(page_size, page_size);
            {
                let test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
                assert!(test.x == 0.0);
                assert!(test.y == 0.0);
            }
        }

        #[test]
        fn alloc_zeroed() {
            let page_size = Platform::get_page_size();
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
        fn with_capacity_reserve() {
            let page_size = Platform::get_page_size();
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
        fn reset() {
            let page_size = Platform::get_page_size();
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
        fn reset_without_shrink() {
            let page_size = Platform::get_page_size();
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
        fn reset_and_shrink() {
            let page_size = Platform::get_page_size();
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

        /// A test for shrinking to the reserve size after using all reserved
        /// memory
        #[test]
        fn shrink_to_reserve_size() {
            let page_size = Platform::get_page_size();
            let reserve_size = 2 * page_size;
            let mut arena =
                DynamicArena::with_capacity_reserve(page_size, reserve_size);

            fill_arena(
                &mut arena,
                TestStruct {
                    ..Default::default()
                },
            );

            arena.reset_and_shrink(reserve_size);
            assert_eq!(arena.used.get(), 0);
            assert_eq!(arena.committed.get(), arena.reserved);

            // test that we can alloc and write after reset
            fill_arena(&mut arena, TestStruct { x: 1.0, y: -1.0 });
        }

        /// A test for shrinking to the reserve size even if not all of it is
        /// committed. The committed value should not increase
        #[test]
        fn shrink_to_reserve_size_under_committed() {
            let page_size = Platform::get_page_size();
            let reserve_size = 2 * page_size;
            let mut arena =
                DynamicArena::with_capacity_reserve(page_size, reserve_size);

            // only fill up to the current committed size
            while page_size - arena.committed.get() > size_of::<TestStruct>() {
                arena.alloc_zeroed::<TestStruct>().unwrap();
            }

            let old_committed = arena.committed.get();
            arena.reset_and_shrink(reserve_size);
            assert_eq!(arena.used.get(), 0);
            assert_eq!(arena.committed.get(), old_committed);

            // test that we can alloc and write after reset
            fill_arena(&mut arena, TestStruct { x: 1.0, y: -1.0 });
        }

        #[test]
        fn reset_and_shrink_page_offset() {
            let page_size = Platform::get_page_size();
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
        fn shrink_to_zero() {
            let page_size = Platform::get_page_size();
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
        fn alloc_struct_page_size_factor() {
            let page_size = Platform::get_page_size();
            let mut arena =
                DynamicArena::with_capacity_reserve(page_size, 4 * page_size);

            fill_arena::<u8>(&mut arena, 0);
            assert_eq!(arena.used.get(), arena.committed.get());
            assert_eq!(arena.committed.get(), arena.reserved);
        }

        /// test allocating memory that is not a factor of the page size
        #[test]
        fn alloc_struct_not_page_size_factor() {
            let page_size = Platform::get_page_size();
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
        fn alloc_to_capacity() {
            let page_size = Platform::get_page_size();
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
        fn alloc_under_page_size() {
            let page_size = Platform::get_page_size();
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
        fn arena_small_reserves() {
            let page_size = Platform::get_page_size();
            let mut arena =
                DynamicArena::with_capacity_reserve(page_size, page_size / 2);
            arena.reset();
        }

        /// A test for initializing an arena where the reserves are not a multiple
        /// of the page size
        #[test]
        fn arena_fractional_page() {
            let page_size = Platform::get_page_size();
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
        fn alloc_array() {
            let page_size = Platform::get_page_size();
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
        fn alloc_array_non_default() {
            let page_size = Platform::get_page_size();
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
        fn alloc_zeroed_array() {
            let page_size = Platform::get_page_size();
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

        #[test]
        fn alloc_uninitialized_array() {
            let page_size = Platform::get_page_size();
            let arena =
                DynamicArena::with_capacity_reserve(page_size, page_size);

            arena
                .alloc_uninitialized_array::<I32Struct>(
                    arena.reserved / size_of::<I32Struct>(),
                )
                .unwrap();

            assert_eq!(arena.used.get(), arena.committed.get());
            assert_eq!(arena.committed.get(), arena.reserved);
        }
    }

    mod benchmark {
        use super::*;
        use crate::test_common::get_element_count;
        use test::Bencher;

        #[bench]
        fn arena_alloc_no_growth(b: &mut Bencher) {
            let element_count = get_element_count();
            let reserve_size = element_count * size_of::<I32Struct>();
            let mut arena =
                DynamicArena::with_capacity_reserve(reserve_size, reserve_size);

            b.iter(|| {
                let elements = arena
                    .alloc_zeroed_array::<I32Struct>(element_count)
                    .unwrap();
                for (index, element) in elements.iter_mut().enumerate() {
                    element.x = index as i32;
                    element.y = -1 * (index as i32);
                }
                arena.reset();
            });
        }

        #[bench]
        fn arena_alloc_grows_and_shrinks(b: &mut Bencher) {
            let element_count = get_element_count();
            let reserve_size = element_count * size_of::<I32Struct>();
            let mut arena =
                DynamicArena::with_capacity_reserve(0, reserve_size);

            b.iter(|| {
                let elements = arena
                    .alloc_zeroed_array::<I32Struct>(element_count)
                    .unwrap();
                for (index, element) in elements.iter_mut().enumerate() {
                    element.x = index as i32;
                    element.y = -1 * (index as i32);
                }
                arena.reset_and_shrink(0);
            });
        }

        #[bench]
        fn arena_alloc_grows_and_shrinks_partial(b: &mut Bencher) {
            let element_count = get_element_count();
            let reserve_size = element_count * size_of::<I32Struct>();
            let mut arena =
                DynamicArena::with_capacity_reserve(0, reserve_size);

            let shrink_to = reserve_size / 2;
            b.iter(|| {
                let elements = arena
                    .alloc_zeroed_array::<I32Struct>(element_count)
                    .unwrap();
                for (index, element) in elements.iter_mut().enumerate() {
                    element.x = index as i32;
                    element.y = -1 * (index as i32);
                }
                arena.reset_and_shrink(shrink_to);
            });
        }

        #[bench]
        fn arena_alloc_shrink_no_growth(b: &mut Bencher) {
            let element_count = get_element_count();
            let reserve_size = element_count * size_of::<I32Struct>();
            let mut arena =
                DynamicArena::with_capacity_reserve(0, reserve_size);

            let shrink_to = reserve_size;
            b.iter(|| {
                let elements = arena
                    .alloc_zeroed_array::<I32Struct>(element_count)
                    .unwrap();
                for (index, element) in elements.iter_mut().enumerate() {
                    element.x = index as i32;
                    element.y = -1 * (index as i32);
                }
                arena.reset_and_shrink(shrink_to);
            });
        }

        fn mutate_mixed_data(
            a: &mut I32Struct,
            b: &mut LargerStruct,
            c: &mut MixedStruct,
            d: &mut SmallerStruct,
            e: &mut SmallStruct,
            f: &mut MixedStruct,
            g: &mut I32Struct,
        ) {
            a.x += 2;
            b.y += 2;
            c.c += 2;
            d.x += 2;
            e.y += 2;
            f.a += 2;
            g.x += 2;
        }

        #[bench]
        fn arena_alloc_mixed(b: &mut Bencher) {
            let element_count = get_element_count();
            let reserve_size = element_count * size_of::<MixedStruct>();
            let mut arena =
                DynamicArena::with_capacity_reserve(reserve_size, reserve_size);
            b.iter(|| {
                let a = arena.alloc_zeroed::<I32Struct>().unwrap();
                let b = arena.alloc_zeroed::<LargerStruct>().unwrap();
                let b1 = arena
                    .alloc_zeroed_array::<I32Struct>(element_count)
                    .unwrap();
                let c = arena.alloc_zeroed::<MixedStruct>().unwrap();
                let d = arena.alloc_zeroed::<SmallerStruct>().unwrap();
                let e = arena.alloc_zeroed::<SmallStruct>().unwrap();
                let f = arena.alloc_zeroed::<MixedStruct>().unwrap();
                let g = arena.alloc_zeroed::<I32Struct>().unwrap();

                mutate_mixed_data(a, b, c, d, e, f, g);
                g.x += b1.len() as i32;

                arena.reset();
            });
        }
    }
}
