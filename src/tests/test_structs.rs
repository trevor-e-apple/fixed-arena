use std::cmp::PartialEq;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestStruct {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LargerStruct {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct I32Struct {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmallerStruct {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmallStruct {
    pub x: i8,
    pub y: i8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixedStruct {
    pub a: i64,
    pub b: i32,
    pub c: i16,
    pub d: i8,
    pub e: f64,
    pub f: f32,
    pub g: u16,
    pub h: u8,
}
