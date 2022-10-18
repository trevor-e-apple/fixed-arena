use core::{cell::Cell, ptr};
use std::{
    alloc::{alloc, dealloc, Layout},
    slice,
    string::String,
};

pub struct FixedArena {
    base: *mut u8,
    used: Cell<usize>,
    capacity: usize,
}

#[derive(Debug, PartialEq)]
pub enum AllocError {
    AtCapacity,
}

// TODO: inline functions?
impl FixedArena {
    // TODO: document me
    pub fn with_capacity(capacity: usize) -> FixedArena {
        // TODO: remove magic alignment
        let layout = Layout::from_size_align(capacity, 4)
            .expect("Bad arguments for layout");
        let base = unsafe { alloc(layout) };
        FixedArena {
            base,
            capacity,
            used: Cell::new(0),
        }
    }

    // TODO: document me
    pub fn with_base(base: *mut u8, capacity: usize) -> FixedArena {
        FixedArena {
            base,
            used: Cell::new(0),
            capacity,
        }
    }

    // TODO: document me
    fn get_alloc_ptr_with_layout(
        &self,
        layout: Layout,
    ) -> Result<*mut u8, AllocError> {
        let result: *mut u8 =
            unsafe { self.base.offset(self.used.get() as isize) };
        self.used.set(self.used.get() + layout.size());
        if self.used.get() <= self.capacity {
            Ok(result)
        } else {
            Err(AllocError::AtCapacity)
        }
    }

    // TODO: document me
    fn get_alloc_ptr<T>(&self) -> Result<(*mut u8, Layout), AllocError> {
        let layout = Layout::new::<T>();
        let pointer = self.get_alloc_ptr_with_layout(layout)?;
        Ok((pointer, layout))
    }

    // TODO: document me
    pub fn alloc<T>(&self, val: T) -> Result<&mut T, AllocError> {
        let (pointer, _) = self.get_alloc_ptr::<T>()?;
        unsafe {
            let result = pointer as *mut T;
            ptr::write(result, val);
            Ok(&mut *result)
        }
    }

    // TODO: document me
    pub fn alloc_zeroed<T>(&self) -> Result<&mut T, AllocError> {
        let (pointer, layout) = self.get_alloc_ptr::<T>()?;
        unsafe {
            let result = pointer as *mut T;
            ptr::write_bytes(pointer, 0, layout.size());
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
        let layout =
            Layout::from_size_align(self.capacity, 4).expect("Layout failed");
        unsafe {
            dealloc(self.base, layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cmp::PartialEq, mem::size_of};

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct TestStruct {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct LargerStruct {
        x: i64,
        y: i64,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct I32Struct {
        x: i32,
        y: i32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct SmallerStruct {
        x: i16,
        y: i16,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct SmallStruct {
        x: i8,
        y: i8,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct MixedStruct {
        a: i64,
        b: i32,
        c: i16,
        d: i8,
        e: f64,
        f: f32,
        g: u16,
        h: u8,
    }

    // TODO: document me
    #[test]
    fn test_init_with_base() {
        todo!();
    }

    // TODO: document me
    #[test]
    fn test_async() {
        todo!();
    }

    // TODO: document me
    #[test]
    fn test_threading() {
        todo!();
    }

    // TODO: document me
    #[test]
    fn test_very_large_arena() {
        todo!();
    }

    // TODO: document me
    #[test]
    fn test_alignment() {
        todo!();
    }

    mod reset_tests {
        use super::*;

        // TODO: document me
        #[test]
        fn test_reset() {
            let mut arena = FixedArena::with_capacity(1024);
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
        fn test_reset_with_mut() {
            let mut arena = FixedArena::with_capacity(1024);

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
        fn test_reset_in_loop() {
            let capacity = 1024;
            let mut arena = FixedArena::with_capacity(capacity);
            for index in 0..capacity {
                let mut test = arena.alloc_zeroed::<TestStruct>().unwrap();
                test.x = 15.0;
                test.y = test.x + (index as f32);
                arena.reset();
            }
        }
    }

    #[cfg(test)]
    mod test_alloc_struct {
        use super::*;

        // TODO: document me
        #[test]
        fn test_basic_allocation() {
            let arena = FixedArena::with_capacity(1024);
            {
                let test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
                assert!(test.x == 0.0);
                assert!(test.y == 0.0);
            }
        }

        // TODO: document me
        #[test]
        fn test_alloc_zeroed() {
            let arena = FixedArena::with_capacity(1024);
            {
                let test = arena.alloc_zeroed::<TestStruct>().unwrap();
                assert!(test.x == 0.0);
                assert!(test.y == 0.0);
            }
        }

        // TODO: document me
        #[test]
        fn test_multiple_allocation() {
            let arena = FixedArena::with_capacity(1024);

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
        fn test_mixed_allocation() {
            let arena = FixedArena::with_capacity(1024);

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
        fn test_at_capacity() {
            let capacity = 1024;
            let count = capacity / size_of::<TestStruct>();
            let arena = FixedArena::with_capacity(capacity);
            for index in 0..count {
                let mut test =
                    arena.alloc(TestStruct { x: 1.0, y: -1.0 }).unwrap();
                test.x = 15.0;
                test.y = test.x + (index as f32);
            }
        }

        // TODO: document me
        #[test]
        fn test_reset_at_capacity() {
            let capacity = 1024;
            let count = capacity;
            let mut arena = FixedArena::with_capacity(capacity);
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
        fn test_over_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
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
        fn test_at_capacity_alloc_zeroed() {
            let capacity = 1024;
            let count = capacity / size_of::<TestStruct>();
            let arena = FixedArena::with_capacity(capacity);
            for index in 0..count {
                let mut test = arena.alloc_zeroed::<TestStruct>().unwrap();
                test.x = 15.0;
                test.y = test.x + (index as f32);
            }
        }

        // TODO: document me
        #[test]
        fn test_reset_at_capacity_alloc_zeroed() {
            let capacity = 1024;
            let count = capacity;
            let mut arena = FixedArena::with_capacity(capacity);
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
        fn test_over_capacity_alloc_zeroed() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
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

    #[cfg(test)]
    mod alloc_array_tests {
        use super::*;

        // TODO: document me
        #[test]
        fn test_alloc_array() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let test_array =
                arena.alloc_array(I32Struct { x: 0, y: 0 }, 8).unwrap();
            alloc_array_common(test_array);
        }

        // TODO: document me
        #[test]
        fn test_alloc_multiple_arrays() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let count = capacity / (2 * size_of::<I32Struct>());
            let test_array_one =
                arena.alloc_array(I32Struct { x: 0, y: 0 }, count).unwrap();
            let test_array_two =
                arena.alloc_array(I32Struct { x: 0, y: 0 }, count).unwrap();

            alloc_multiple_arrays_common(test_array_one, test_array_two);
        }

        // TODO: document me
        #[test]
        fn test_alloc_array_to_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let count = capacity / size_of::<I32Struct>();
            let test_array =
                arena.alloc_array(I32Struct { x: 1, y: -1 }, count).unwrap();

            alloc_array_to_capacity_common(test_array, capacity);
        }

        // TODO: document me
        #[test]
        fn test_alloc_array_over_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let result = arena.alloc_array(
                TestStruct { x: 0.0, y: 0.0 },
                capacity / size_of::<TestStruct>() + 1,
            );
            alloc_array_over_capacity_common(result);
        }

        // TODO: document me
        #[test]
        fn test_alloc_array_to_capacity_reset() {
            let capacity = 1024;
            let count = capacity / size_of::<TestStruct>();
            let mut arena = FixedArena::with_capacity(capacity);
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
        fn test_alloc_zeroed_array() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let test_array = arena.alloc_zeroed_array::<I32Struct>(8).unwrap();

            verify_i32_struct_array(test_array, 0, 0);

            alloc_array_common(test_array);
        }

        #[test]
        fn test_alloc_multiple_zeroed_array() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
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
        fn test_alloc_zeroed_array_to_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let count = capacity / size_of::<I32Struct>();
            let array = arena.alloc_zeroed_array::<I32Struct>(count).unwrap();
            verify_i32_struct_array(array, 0, 0);

            alloc_array_to_capacity_common(array, capacity);
        }

        #[test]
        fn test_alloc_zeroed_array_over_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let count = (capacity / size_of::<I32Struct>()) + 1;
            match arena.alloc_zeroed_array::<I32Struct>(count) {
                Ok(_) => assert!(false),
                Err(err) => assert_eq!(err, AllocError::AtCapacity),
            };
        }

        #[test]
        fn test_alloc_zeroed_array_to_capacity_reset() {
            let capacity = 1024;
            let mut arena = FixedArena::with_capacity(capacity);
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
        fn test_alloc_unitialized_array() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let array =
                arena.alloc_uninitialized_array::<I32Struct>(8).unwrap();

            alloc_array_common(array);
        }

        #[test]
        fn test_alloc_multiple_unitialized_array() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let count = capacity / (2 * size_of::<I32Struct>());
            let test_array_one =
                arena.alloc_uninitialized_array::<I32Struct>(count).unwrap();
            let test_array_two =
                arena.alloc_uninitialized_array::<I32Struct>(count).unwrap();

            alloc_multiple_arrays_common(test_array_one, test_array_two);
        }

        #[test]
        fn test_alloc_unitialized_array_to_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let count = capacity / size_of::<I32Struct>();
            let array =
                arena.alloc_uninitialized_array::<I32Struct>(count).unwrap();

            alloc_array_to_capacity_common(array, capacity);
        }

        #[test]
        fn test_alloc_unitialized_array_over_capacity() {
            let capacity = 1024;
            let arena = FixedArena::with_capacity(capacity);
            let count = capacity / size_of::<I32Struct>() + 1;
            let result = arena.alloc_uninitialized_array::<I32Struct>(count);

            alloc_array_over_capacity_common(result);
        }

        #[test]
        fn test_alloc_unitialized_array_to_capacity_reset() {
            let capacity = 1024;
            let mut arena = FixedArena::with_capacity(capacity);
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
}
