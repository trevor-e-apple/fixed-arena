use core::{cell::Cell, ptr};
use std::{
    alloc::{alloc, dealloc, Layout, LayoutError},
    slice,
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
    pub fn with_capacity(capacity: usize) -> Result<FixedArena, LayoutError> {
        // TODO: remove magic alignment
        let layout = Layout::from_size_align(capacity, 4)?;
        let base = unsafe { alloc(layout) };
        Ok(FixedArena {
            base,
            capacity,
            used: Cell::new(0),
        })
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

// TODO: move this to the tests directory?
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
    fn test_basic_allocation() {
        let arena = FixedArena::with_capacity(1024).unwrap();
        {
            let test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
            assert!(test.x == 0.0);
            assert!(test.y == 0.0);
        }
    }

    // TODO: document me
    #[test]
    fn test_init_with_base() {
        todo!();
    }

    // TODO: document me
    #[test]
    fn test_reset() {
        let mut arena = FixedArena::with_capacity(1024).unwrap();
        {
            arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
        }
        arena.reset();
        assert!(arena.used.get() == 0);
        {
            let test_two = arena.alloc(TestStruct { x: 1.0, y: 2.0 }).unwrap();

            assert!(test_two.x == 1.0);
            assert!(test_two.y == 2.0);
        }
    }

    // TODO: document me
    #[test]
    fn test_reset_with_mut() {
        let mut arena = FixedArena::with_capacity(1024).unwrap();

        {
            let mut test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
            assert!(test.x == 0.0);
            test.x = 1.0;
            assert!(test.x == 1.0);
        }

        arena.reset();
        assert!(arena.used.get() == 0);

        {
            let test_two = arena.alloc(TestStruct { x: 1.0, y: 2.0 }).unwrap();

            assert!(test_two.x == 1.0);
            assert!(test_two.y == 2.0);
        }
    }

    // TODO: document me
    #[test]
    fn test_alloc_zeroed() {
        let arena = FixedArena::with_capacity(1024).unwrap();
        {
            let test = arena.alloc_zeroed::<TestStruct>().unwrap();
            assert!(test.x == 0.0);
            assert!(test.y == 0.0);
        }
    }

    // TODO: document me
    #[test]
    fn test_alignment() {
        todo!();
    }

    // TODO: document me
    #[test]
    fn test_multiple_allocation() {
        let arena = FixedArena::with_capacity(1024).unwrap();

        {
            let test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
            let test_two = arena.alloc(TestStruct { x: 1.0, y: 2.0 }).unwrap();

            assert!(test.x == 0.0);
            assert!(test.y == 0.0);
            assert!(test_two.x == 1.0);
            assert!(test_two.y == 2.0);
        }
    }

    // TODO: document me
    #[test]
    fn test_reset_in_loop() {
        let capacity = 1024;
        let mut arena = FixedArena::with_capacity(capacity).unwrap();
        for index in 0..capacity {
            let mut test = arena.alloc_zeroed::<TestStruct>().unwrap();
            test.x = 15.0;
            test.y = test.x + (index as f32);
            arena.reset();
        }
    }

    fn alloc_and_check<T>(arena: &FixedArena, val: T) -> &mut T
    where
        T: PartialEq + Copy,
    {
        let result = arena.alloc(val).unwrap();
        assert!(*result == val);

        result
    }

    // TODO: document me
    #[test]
    fn test_mixed_allocation() {
        let arena = FixedArena::with_capacity(1024).unwrap();

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
        let arena = FixedArena::with_capacity(capacity).unwrap();
        for index in 0..count {
            let mut test = arena.alloc(TestStruct { x: 1.0, y: -1.0 }).unwrap();
            test.x = 15.0;
            test.y = test.x + (index as f32);
        }
    }

    // TODO: document me
    #[test]
    fn test_reset_at_capacity() {
        let capacity = 1024;
        let count = capacity;
        let mut arena = FixedArena::with_capacity(capacity).unwrap();
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
        let arena = FixedArena::with_capacity(capacity).unwrap();
        let count = capacity / size_of::<TestStruct>();
        for index in 0..count {
            let mut test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
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
        let arena = FixedArena::with_capacity(capacity).unwrap();
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
        let mut arena = FixedArena::with_capacity(capacity).unwrap();
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
        let arena = FixedArena::with_capacity(capacity).unwrap();
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

    // TODO: document me
    #[test]
    fn test_alloc_array() {
        let capacity = 1024;
        let arena = FixedArena::with_capacity(capacity).unwrap();
        let test_array =
            arena.alloc_array(TestStruct { x: 0.0, y: 0.0 }, 8).unwrap();
        for test_value in test_array {
            test_value.x = 1.0;
            test_value.y = -1.0;
        }
    }

    // TODO: document me
    fn fill_test_arrays(
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

    // TODO: document me
    #[test]
    fn test_alloc_multiple_arrays() {
        let capacity = 1024;
        let arena = FixedArena::with_capacity(capacity).unwrap();
        let count = capacity / (2 * size_of::<I32Struct>());
        let test_array_one =
            arena.alloc_array(I32Struct { x: 0, y: 0 }, count).unwrap();
        let test_array_two =
            arena.alloc_array(I32Struct { x: 0, y: 0 }, count).unwrap();

        fill_test_arrays(test_array_one, test_array_two);
    }

    // TODO: document me
    #[test]
    fn test_alloc_array_to_capacity() {
        let capacity = 1024;
        let arena = FixedArena::with_capacity(capacity).unwrap();
        let test_array = arena
            .alloc_array(
                TestStruct { x: 0.0, y: 1.0 },
                capacity / size_of::<TestStruct>(),
            )
            .unwrap();

        // just in case we change the TestStruct def, be sure that we actually
        // test to the complete capacity of the arena
        assert_eq!(capacity % size_of::<TestStruct>(), 0);

        for test_value in test_array {
            test_value.x = 1.0;
            test_value.y = -1.0;
        }
    }

    // TODO: document me
    #[test]
    fn test_alloc_array_over_capacity() {
        let capacity = 1024;
        let arena = FixedArena::with_capacity(capacity).unwrap();
        let result = arena.alloc_array(
            TestStruct { x: 0.0, y: 0.0 },
            capacity / size_of::<TestStruct>() + 1,
        );
        match result {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err, AllocError::AtCapacity),
        };
    }

    // TODO: document me
    #[test]
    fn test_alloc_array_to_capacity_reset() {
        let capacity = 1024;
        let count = capacity / size_of::<TestStruct>();
        let mut arena = FixedArena::with_capacity(capacity).unwrap();
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
        let arena = FixedArena::with_capacity(capacity).unwrap();
        let test_array = arena.alloc_zeroed_array::<TestStruct>(8).unwrap();
        for test_value in test_array.iter() {
            assert_eq!(test_value.x, 0.0);
            assert_eq!(test_value.y, 0.0);
        }
        for test_value in test_array.iter_mut() {
            test_value.x = 1.0;
            test_value.y = -1.0;
        }

        // verify that the write was successful
        for test_value in test_array.iter() {
            assert_eq!(test_value.x, 1.0);
            assert_eq!(test_value.y, -1.0);
        }
    }

    fn verify_zeroed_i32_struct_array(array: &[I32Struct]) {
        for element in array {
            assert_eq!(element.x, 0);
            assert_eq!(element.y, 0);
        }
    }

    #[test]
    fn test_alloc_multiple_zeroed_array() {
        let capacity = 1024;
        let arena = FixedArena::with_capacity(capacity).unwrap();
        let count = capacity / (2 * size_of::<I32Struct>());
        let test_array_one =
            arena.alloc_zeroed_array::<I32Struct>(count).unwrap();
        let test_array_two =
            arena.alloc_zeroed_array::<I32Struct>(count).unwrap();

        verify_zeroed_i32_struct_array(&test_array_one);
        verify_zeroed_i32_struct_array(&test_array_two);

        fill_test_arrays(test_array_one, test_array_two);
    }

    #[test]
    fn test_alloc_zeroed_array_at_capacity() {
        let capacity = 1024;
        let arena = FixedArena::with_capacity(capacity).unwrap();
        let count = capacity / size_of::<I32Struct>();
        let array = arena.alloc_zeroed_array::<I32Struct>(count).unwrap();
        verify_zeroed_i32_struct_array(array);

        for (index, element) in array.iter_mut().enumerate() {
            element.x = index as i32;
            element.y = -1 * (index as i32);
        }

        for (index, element) in array.iter().enumerate() {
            assert_eq!(element.x, index as i32);
            assert_eq!(element.y, -1 * (index as i32));
        }
    }

    #[test]
    fn test_alloc_zeroed_array_over_capacity() {
        let capacity = 1024;
        let arena = FixedArena::with_capacity(capacity).unwrap();
        let count = (capacity / size_of::<I32Struct>()) + 1;
        match arena.alloc_zeroed_array::<I32Struct>(count) {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err, AllocError::AtCapacity),
        };
    }

    #[test]
    fn test_alloc_zeroed_array_to_capacity_reset() {
        let capacity = 1024;
        let mut arena = FixedArena::with_capacity(capacity).unwrap();
        let count = capacity / size_of::<I32Struct>();
        arena.alloc_zeroed_array::<I32Struct>(count).unwrap();

        // should fail
        match arena.alloc_zeroed_array::<I32Struct>(count) {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err, AllocError::AtCapacity),
        };

        arena.reset();

        arena.alloc_zeroed_array::<I32Struct>(count).unwrap();
    }

    #[test]
    fn test_alloc_unitialized_array() {
        todo!();
    }

    #[test]
    fn test_alloc_multiple_unitialized_array() {
        todo!();
    }

    #[test]
    fn test_alloc_unitialized_array_at_capacity() {
        todo!();
    }

    #[test]
    fn test_alloc_unitialized_array_over_capacity() {
        todo!();
    }

    #[test]
    fn test_alloc_unitialized_array_to_capacity_reset() {
        todo!();
    }

    // TODO: document me
    #[test]
    fn test_alloc_string() {
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
}
