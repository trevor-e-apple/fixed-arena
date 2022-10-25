use core::{cell::Cell, ptr};
use std::{
    alloc::{alloc, dealloc, Layout},
    slice,
};

use crate::errors::AllocError;

pub struct FixedArena {
    base: *mut u8,
    base_align: usize,
    used: Cell<usize>,
    capacity: usize,
}

// TODO: inline functions?
impl FixedArena {
    // TODO: document me
    pub fn with_capacity(capacity: usize, align: usize) -> FixedArena {
        // TODO: remove magic alignment
        let layout = Layout::from_size_align(capacity, align)
            .expect("Bad arguments for layout");
        let base = unsafe { alloc(layout) };
        FixedArena {
            base,
            base_align: align,
            capacity,
            used: Cell::new(0),
        }
    }

    // TODO: document me
    fn get_alloc_ptr_with_layout(
        &self,
        layout: Layout,
    ) -> Result<*mut u8, AllocError> {
        let new_used = self.used.get() + layout.size();
        if new_used <= self.capacity {
            let result: *mut u8 =
                unsafe { self.base.offset(self.used.get() as isize) };
            self.used.set(new_used);
            Ok(result)
        } else {
            Err(AllocError::AtCapacity)
        }
    }

    // TODO: document me
    fn get_alloc_ptr<T>(&self) -> Result<*mut u8, AllocError> {
        let layout = Layout::new::<T>();
        let pointer = self.get_alloc_ptr_with_layout(layout)?;
        Ok(pointer)
    }

    // TODO: document me
    pub fn alloc<T>(&self, val: T) -> Result<&mut T, AllocError> {
        let pointer = self.get_alloc_ptr::<T>()?;
        unsafe {
            let result = pointer as *mut T;
            ptr::write(result, val);
            Ok(&mut *result)
        }
    }

    // TODO: document me
    pub fn alloc_zeroed<T>(&self) -> Result<&mut T, AllocError> {
        let pointer = self.get_alloc_ptr::<T>()?;
        unsafe {
            let result = pointer as *mut T;
            ptr::write_bytes(result, 0, 1);
            Ok(&mut *result)
        }
    }

    // TODO: document me
    /// alloc_array does perform a cast from usize to isize, and will panic if
    /// the coutn value does not fit in isize
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
        let pointer = self.get_alloc_ptr_with_layout(layout)?;

        let result: &mut [T];
        unsafe {
            let pointer = pointer as *mut T;
            let isize_count = count as isize;
            for index in 0..isize_count {
                ptr::write(pointer.offset(index), val.clone());
            }
            result = slice::from_raw_parts_mut(pointer as *mut T, count);
        }

        Ok(result)
    }

    // TODO: document me
    pub fn alloc_zeroed_array<T>(
        &self,
        count: usize,
    ) -> Result<&mut [T], AllocError> {
        let layout =
            Layout::array::<T>(count).expect("Bad count value for array");
        let pointer = self.get_alloc_ptr_with_layout(layout)?;
        unsafe {
            ptr::write_bytes(pointer as *mut T, 0, count);
            let result = slice::from_raw_parts_mut(pointer as *mut T, count);
            Ok(result)
        }
    }

    // TODO: more documentation, examples
    /// Allocs an array of T. The size of the array is count.
    /// The data in the array is uninitialized
    pub fn alloc_uninitialized_array<T>(
        &self,
        count: usize,
    ) -> Result<&mut [T], AllocError> {
        let layout =
            Layout::array::<T>(count).expect("Bad count value for array");
        let pointer = self.get_alloc_ptr_with_layout(layout)?;
        unsafe {
            let result = slice::from_raw_parts_mut(pointer as *mut T, count);
            Ok(result)
        }
    }

    /// Because the alloc method immutably borrows self and reset mutably
    /// borrows self, a call to reset will invalidate all previous values that
    /// were allocated using the alloc method. This is because Rust will not
    /// allow an immutable borrow of self to exist past a mutable borrow of
    /// self.
    // TODO: more documentation, examples
    pub fn reset(&mut self) {
        self.used.set(0);
    }
}

impl Drop for FixedArena {
    // TODO: document me
    fn drop(&mut self) {
        // TODO: remove magic alignment
        let layout = Layout::from_size_align(self.capacity, self.base_align)
            .expect("Layout failed");
        unsafe {
            dealloc(self.base, layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cmp::PartialEq, mem::size_of};
    const DEFAULT_ALIGN: usize = 4;

    use crate::test_common::{
        I32Struct, LargerStruct, MixedStruct, SmallStruct, SmallerStruct,
        TestStruct,
    };

    mod reset {
        use super::*;

        // TODO: document me
        #[test]
        fn reset() {
            let mut arena = FixedArena::with_capacity(1024, DEFAULT_ALIGN);
            {
                arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
            }
            arena.reset();
            assert!(arena.used.get() == 0);
            {
                let test_two =
                    arena.alloc(TestStruct { x: 1.0, y: 2.0 }).unwrap();

                assert!(test_two.x == 1.0);
                assert!(test_two.y == 2.0);
            }
        }

        // TODO: document me
        #[test]
        fn reset_with_mut() {
            let mut arena = FixedArena::with_capacity(1024, DEFAULT_ALIGN);

            {
                let mut test =
                    arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
                assert!(test.x == 0.0);
                test.x = 1.0;
                assert!(test.x == 1.0);
            }

            arena.reset();
            assert!(arena.used.get() == 0);

            {
                let test_two =
                    arena.alloc(TestStruct { x: 1.0, y: 2.0 }).unwrap();

                assert!(test_two.x == 1.0);
                assert!(test_two.y == 2.0);
            }
        }

        // TODO: document me
        #[test]
        fn reset_in_loop() {
            let capacity = 1024;
            let mut arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            for index in 0..capacity {
                let mut test = arena.alloc_zeroed::<TestStruct>().unwrap();
                test.x = 15.0;
                test.y = test.x + (index as f32);
                arena.reset();
            }
        }
    }

    mod alloc_struct {
        use super::*;

        // TODO: document me
        #[test]
        fn basic_allocation() {
            let arena = FixedArena::with_capacity(1024, DEFAULT_ALIGN);
            {
                let test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
                assert!(test.x == 0.0);
                assert!(test.y == 0.0);
            }
        }

        // TODO: document me
        #[test]
        fn alloc_zeroed() {
            let arena = FixedArena::with_capacity(1024, DEFAULT_ALIGN);
            {
                let test = arena.alloc_zeroed::<TestStruct>().unwrap();
                assert!(test.x == 0.0);
                assert!(test.y == 0.0);
            }
        }

        // TODO: document me
        #[test]
        fn multiple_allocation() {
            let arena = FixedArena::with_capacity(1024, DEFAULT_ALIGN);

            {
                let test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
                let test_two =
                    arena.alloc(TestStruct { x: 1.0, y: 2.0 }).unwrap();

                assert!(test.x == 0.0);
                assert!(test.y == 0.0);
                assert!(test_two.x == 1.0);
                assert!(test_two.y == 2.0);
            }
        }

        // TODO: document me
        #[test]
        fn mixed_allocation() {
            let arena = FixedArena::with_capacity(1024, DEFAULT_ALIGN);

            let first = TestStruct { x: 1.0, y: -1.0 };
            let first_result = alloc_and_check(&arena, first);

            let second = LargerStruct {
                x: (1 << 42),
                y: -1 * (1 << 42),
            };
            let second_result = alloc_and_check(&arena, second);

            let third = SmallerStruct {
                x: 1 << 9,
                y: -1 * (1 << 9),
            };
            let third_result = alloc_and_check(&arena, third);

            let fourth = SmallStruct { x: 127, y: -1 };
            let fourth_result = alloc_and_check(&arena, fourth);

            let fifth = MixedStruct {
                a: 1 << 33,
                b: 1 << 17,
                c: 1 << 9,
                d: 127,
                e: 1.000454846,
                f: -1.000454846,
                g: 0xFFFF,
                h: 0xFF,
            };
            let fifth_result = alloc_and_check(&arena, fifth);

            assert!(first == *first_result);
            assert!(second == *second_result);
            assert!(third == *third_result);
            assert!(fourth == *fourth_result);
            assert!(fifth == *fifth_result);
        }

        // TODO: document me
        #[test]
        fn at_capacity() {
            let capacity = 1024;
            let count = capacity / size_of::<TestStruct>();
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            for index in 0..count {
                let mut test =
                    arena.alloc(TestStruct { x: 1.0, y: -1.0 }).unwrap();
                test.x = 15.0;
                test.y = test.x + (index as f32);
            }
        }

        // TODO: document me
        #[test]
        fn reset_at_capacity() {
            let capacity = 1024;
            let count = capacity;
            let mut arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            for index in 0..count {
                let test = match arena.alloc(TestStruct { x: 1.0, y: -1.0 }) {
                    Ok(result) => result,
                    Err(error) => match error {
                        // if we're at capacity, reset and retry, panic if alloc
                        // fails again
                        AllocError::AtCapacity => {
                            arena.reset();
                            arena.alloc(TestStruct { x: 1.0, y: -1.0 }).unwrap()
                        }
                    },
                };
                test.x = 15.0;
                test.y = test.x + (index as f32);
            }
        }

        // TODO: document me
        #[test]
        fn over_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / size_of::<TestStruct>();
            for index in 0..count {
                let mut test =
                    arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
                test.x = 15.0;
                test.y = test.x + (index as f32);
            }
            match arena.alloc(TestStruct { x: 0.0, y: 0.0 }) {
                Ok(_) => assert!(false),
                Err(err) => assert_eq!(err, AllocError::AtCapacity),
            };
        }

        // TODO: document me
        #[test]
        fn at_capacity_alloc_zeroed() {
            let capacity = 1024;
            let count = capacity / size_of::<TestStruct>();
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            for index in 0..count {
                let mut test = arena.alloc_zeroed::<TestStruct>().unwrap();
                test.x = 15.0;
                test.y = test.x + (index as f32);
            }
        }

        // TODO: document me
        #[test]
        fn reset_at_capacity_alloc_zeroed() {
            let capacity = 1024;
            let count = capacity;
            let mut arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            for index in 0..count {
                let test = match arena.alloc_zeroed::<TestStruct>() {
                    Ok(result) => result,
                    Err(error) => match error {
                        // if we're at capacity, reset and retry, panic if alloc
                        // fails again
                        AllocError::AtCapacity => {
                            arena.reset();
                            arena.alloc_zeroed::<TestStruct>().unwrap()
                        }
                    },
                };
                test.x = 15.0;
                test.y = test.x + (index as f32);
            }
        }

        // TODO: document me
        #[test]
        fn over_capacity_alloc_zeroed() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / size_of::<TestStruct>();
            for index in 0..count {
                let mut test = arena.alloc_zeroed::<TestStruct>().unwrap();
                test.x = 15.0;
                test.y = test.x + (index as f32);
            }

            match arena.alloc_zeroed::<TestStruct>() {
                Ok(_) => assert!(false),
                Err(err) => assert_eq!(err, AllocError::AtCapacity),
            };
        }

        fn alloc_and_check<T>(arena: &FixedArena, val: T) -> &mut T
        where
            T: PartialEq + Copy,
        {
            let result = arena.alloc(val).unwrap();
            assert!(*result == val);

            result
        }
    }

    mod alloc_array {
        use super::*;

        // TODO: document me
        #[test]
        fn alloc_array() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let test_array =
                arena.alloc_array(I32Struct { x: 0, y: 0 }, 8).unwrap();
            alloc_array_common(test_array);
        }

        // TODO: document me
        #[test]
        fn alloc_multiple_arrays() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / (2 * size_of::<I32Struct>());
            let test_array_one =
                arena.alloc_array(I32Struct { x: 0, y: 0 }, count).unwrap();
            let test_array_two =
                arena.alloc_array(I32Struct { x: 0, y: 0 }, count).unwrap();

            alloc_multiple_arrays_common(test_array_one, test_array_two);
        }

        // TODO: document me
        #[test]
        fn alloc_array_to_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / size_of::<I32Struct>();
            let test_array =
                arena.alloc_array(I32Struct { x: 1, y: -1 }, count).unwrap();

            alloc_array_to_capacity_common(test_array, capacity);
        }

        // TODO: document me
        #[test]
        fn alloc_array_over_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let result = arena.alloc_array(
                TestStruct { x: 0.0, y: 0.0 },
                capacity / size_of::<TestStruct>() + 1,
            );
            alloc_array_over_capacity_common(result);
        }

        // TODO: document me
        #[test]
        fn alloc_array_to_capacity_reset() {
            let capacity = 1024;
            let count = capacity / size_of::<TestStruct>();
            let mut arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            arena
                .alloc_array(TestStruct { x: 0.0, y: 0.0 }, count)
                .unwrap();

            // attempt to alloc another array, should fail
            match arena.alloc_array(TestStruct { x: 0.0, y: 0.0 }, count) {
                Ok(_) => assert!(false),
                Err(err) => assert!(err == AllocError::AtCapacity),
            };

            arena.reset();

            // second attempt should succeed
            arena
                .alloc_array(TestStruct { x: 0.0, y: 0.0 }, count)
                .unwrap();
        }

        #[test]
        fn alloc_zeroed_array() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let test_array = arena.alloc_zeroed_array::<I32Struct>(8).unwrap();

            verify_i32_struct_array(test_array, 0, 0);

            alloc_array_common(test_array);
        }

        #[test]
        fn alloc_multiple_zeroed_array() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / (2 * size_of::<I32Struct>());
            let test_array_one =
                arena.alloc_zeroed_array::<I32Struct>(count).unwrap();
            let test_array_two =
                arena.alloc_zeroed_array::<I32Struct>(count).unwrap();

            verify_i32_struct_array(&test_array_one, 0, 0);
            verify_i32_struct_array(&test_array_two, 0, 0);

            alloc_multiple_arrays_common(test_array_one, test_array_two);
        }

        #[test]
        fn alloc_zeroed_array_to_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / size_of::<I32Struct>();
            let array = arena.alloc_zeroed_array::<I32Struct>(count).unwrap();
            verify_i32_struct_array(array, 0, 0);

            alloc_array_to_capacity_common(array, capacity);
        }

        #[test]
        fn alloc_zeroed_array_over_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = (capacity / size_of::<I32Struct>()) + 1;
            match arena.alloc_zeroed_array::<I32Struct>(count) {
                Ok(_) => assert!(false),
                Err(err) => assert_eq!(err, AllocError::AtCapacity),
            };
        }

        #[test]
        fn alloc_zeroed_array_to_capacity_reset() {
            let capacity = 1024;
            let mut arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / size_of::<I32Struct>();
            arena.alloc_zeroed_array::<I32Struct>(count).unwrap();

            // should fail
            match arena.alloc_zeroed_array::<I32Struct>(count) {
                Ok(_) => assert!(false),
                Err(err) => assert_eq!(err, AllocError::AtCapacity),
            };

            arena.reset();

            let array = arena.alloc_zeroed_array::<I32Struct>(count).unwrap();
            verify_i32_struct_array(array, 0, 0);
        }

        #[test]
        fn alloc_unitialized_array() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let array =
                arena.alloc_uninitialized_array::<I32Struct>(8).unwrap();

            alloc_array_common(array);
        }

        #[test]
        fn alloc_multiple_unitialized_array() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / (2 * size_of::<I32Struct>());
            let test_array_one =
                arena.alloc_uninitialized_array::<I32Struct>(count).unwrap();
            let test_array_two =
                arena.alloc_uninitialized_array::<I32Struct>(count).unwrap();

            alloc_multiple_arrays_common(test_array_one, test_array_two);
        }

        #[test]
        fn alloc_unitialized_array_to_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / size_of::<I32Struct>();
            let array =
                arena.alloc_uninitialized_array::<I32Struct>(count).unwrap();

            alloc_array_to_capacity_common(array, capacity);
        }

        #[test]
        fn alloc_unitialized_array_over_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / size_of::<I32Struct>() + 1;
            let result = arena.alloc_uninitialized_array::<I32Struct>(count);

            alloc_array_over_capacity_common(result);
        }

        #[test]
        fn alloc_unitialized_array_to_capacity_reset() {
            let capacity = 1024;
            let mut arena = FixedArena::with_capacity(capacity, DEFAULT_ALIGN);
            let count = capacity / size_of::<I32Struct>();
            arena.alloc_uninitialized_array::<I32Struct>(count).unwrap();

            // should fail
            match arena.alloc_uninitialized_array::<I32Struct>(count) {
                Ok(_) => assert!(false),
                Err(err) => assert_eq!(err, AllocError::AtCapacity),
            };

            arena.reset();

            // second attempt should succeed
            arena.alloc_uninitialized_array::<I32Struct>(count).unwrap();
        }

        fn alloc_array_common(array: &mut [I32Struct]) {
            const X_VALUE: i32 = 1;
            const Y_VALUE: i32 = -1;

            for element in array.iter_mut() {
                element.x = X_VALUE;
                element.y = Y_VALUE;
            }

            verify_i32_struct_array(array, 1, -1);
        }

        fn alloc_array_to_capacity_common(
            test_array: &mut [I32Struct],
            capacity: usize,
        ) {
            assert_eq!(capacity % size_of::<I32Struct>(), 0);

            const X_VALUE: i32 = 1;
            const Y_VALUE: i32 = -1;

            for element in test_array.iter_mut() {
                element.x = X_VALUE;
                element.y = Y_VALUE;
            }

            verify_i32_struct_array(test_array, X_VALUE, Y_VALUE);
        }

        // TODO: document me
        fn alloc_multiple_arrays_common(
            test_array_one: &mut [I32Struct],
            test_array_two: &mut [I32Struct],
        ) {
            const ARRAY_ONE_X_VALUE: i32 = 0x7FFFFFFF;
            const ARRAY_ONE_Y_VALUE: i32 = -1;

            const ARRAY_TWO_X_VALUE: i32 = 0x7ABABABA;
            const ARRAY_TWO_Y_VALUE: i32 = -1 * 0x7ABABABA;

            for test_value in test_array_one.iter_mut() {
                test_value.x = ARRAY_ONE_X_VALUE;
                test_value.y = ARRAY_ONE_Y_VALUE;
            }
            for test_value in test_array_two.iter_mut() {
                test_value.x = ARRAY_TWO_X_VALUE;
                test_value.y = ARRAY_TWO_Y_VALUE;
            }

            for test_value in test_array_one.iter() {
                assert_eq!(test_value.x, ARRAY_ONE_X_VALUE);
                assert_eq!(test_value.y, ARRAY_ONE_Y_VALUE);
            }
            for test_value in test_array_two.iter() {
                assert_eq!(test_value.x, ARRAY_TWO_X_VALUE);
                assert_eq!(test_value.y, ARRAY_TWO_Y_VALUE);
            }

            // make sure array_one can't overwrite array_two
            for test_value in test_array_one.iter_mut() {
                test_value.x = ARRAY_ONE_X_VALUE;
                test_value.y = ARRAY_ONE_Y_VALUE;
            }

            for test_value in test_array_one.iter() {
                assert_eq!(test_value.x, ARRAY_ONE_X_VALUE);
                assert_eq!(test_value.y, ARRAY_ONE_Y_VALUE);
            }
            for test_value in test_array_two.iter() {
                assert_eq!(test_value.x, ARRAY_TWO_X_VALUE);
                assert_eq!(test_value.y, ARRAY_TWO_Y_VALUE);
            }
        }

        fn alloc_array_over_capacity_common<T>(result: Result<T, AllocError>) {
            match result {
                Ok(_) => assert!(false),
                Err(err) => assert_eq!(err, AllocError::AtCapacity),
            };
        }

        fn verify_i32_struct_array(
            array: &[I32Struct],
            expected_x_value: i32,
            expected_y_value: i32,
        ) {
            for element in array {
                assert_eq!(element.x, expected_x_value);
                assert_eq!(element.y, expected_y_value);
            }
        }
    }

    mod benchmark {
        use super::*;
        use crate::test_common::get_element_count;
        use test::Bencher;

        #[bench]
        fn arena_alloc(b: &mut Bencher) {
            let element_count = get_element_count();
            let mut arena = FixedArena::with_capacity(
                element_count * size_of::<I32Struct>(),
                DEFAULT_ALIGN,
            );

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
            let mut arena = FixedArena::with_capacity(
                element_count * size_of::<I32Struct>() + 2048,
                DEFAULT_ALIGN,
            );
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
