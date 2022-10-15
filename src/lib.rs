use core::{cell::Cell, ptr};
use std::alloc::{alloc, dealloc, Layout, LayoutError};

pub struct FixedArena {
    base: *mut u8,
    used: Cell<usize>,
    capacity: usize,
}

#[derive(Debug)]
pub enum AllocError {
    AtCapacity,
}

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

    // TODO: initialize with given base

    // TODO: document me
    fn get_alloc_ptr<T>(&self) -> Result<(*mut u8, Layout), AllocError> {
        let layout = Layout::new::<T>();
        let result: *mut u8 =
            unsafe { self.base.offset(self.used.get() as isize) };
        self.used.set(self.used.get() + layout.size());
        if self.used.get() <= self.capacity {
            Ok((result, layout))
        } else {
            Err(AllocError::AtCapacity)
        }
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

    #[test]
    fn test_basic_allocation() {
        let arena = FixedArena::with_capacity(1024).unwrap();
        {
            let test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
            assert!(test.x == 0.0);
            assert!(test.y == 0.0);
        }
    }

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

    #[test]
    fn test_alloc_zeroed() {
        let arena = FixedArena::with_capacity(1024).unwrap();
        {
            let test = arena.alloc_zeroed::<TestStruct>().unwrap();
            assert!(test.x == 0.0);
            assert!(test.y == 0.0);
        }
    }

    #[test]
    fn test_alignment() {
        todo!();
    }

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

    #[test]
    #[should_panic]
    fn test_over_capacity() {
        let capacity = 1024;
        let arena = FixedArena::with_capacity(capacity).unwrap();
        for index in 0..capacity {
            let mut test = arena.alloc(TestStruct { x: 0.0, y: 0.0 }).unwrap();
            test.x = 15.0;
            test.y = test.x + (index as f32);
        }
    }

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

    #[test]
    #[should_panic]
    fn test_over_capacity_alloc_zeroed() {
        let capacity = 1024;
        let arena = FixedArena::with_capacity(capacity).unwrap();
        for index in 0..capacity {
            let mut test = arena.alloc_zeroed::<TestStruct>().unwrap();
            test.x = 15.0;
            test.y = test.x + (index as f32);
        }
    }

    #[test]
    fn test_alloc_array() {
        todo!();
    }

    #[test]
    fn test_alloc_string() {
        todo!();
    }

    #[test]
    fn test_async() {
        todo!();
    }

    #[test]
    fn test_threading() {
        todo!();
    }
}
