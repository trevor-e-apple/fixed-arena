#![feature(test)]
extern crate test;

use std::{mem::size_of, vec::Vec};

use test::Bencher;

use tea_arena::fixed_arena::FixedArena;

mod test_structs;
use crate::test_structs::{
    I32Struct, LargerStruct, MixedStruct, SmallStruct, SmallerStruct,
};

mod test_common;
use crate::test_common::get_element_count;

const DEFAULT_ALIGN: usize = 4;

#[bench]
fn std_alloc_push(b: &mut Bencher) {
    let element_count = get_element_count();
    b.iter(|| {
        let mut elements: Vec<I32Struct> = Vec::new();
        for _ in 0..element_count {
            elements.push(I32Struct { x: 0, y: 0 });
        }
        for (index, element) in elements.iter_mut().enumerate() {
            element.x = index as i32;
            element.y = -1 * (index as i32);
        }
    });
}

// TODO: this is not specific to this arena. move
#[bench]
fn std_alloc(b: &mut Bencher) {
    let element_count = get_element_count();
    b.iter(|| {
        let mut elements: Vec<I32Struct> = Vec::with_capacity(element_count);
        for _ in 0..element_count {
            elements.push(I32Struct { x: 0, y: 0 });
        }
        for (index, element) in elements.iter_mut().enumerate() {
            element.x = index as i32;
            element.y = -1 * (index as i32);
        }
    });
}

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
fn std_alloc_mixed(b: &mut Bencher) {
    let element_count = get_element_count();
    b.iter(|| {
        let mut a: Box<I32Struct> = Box::new(Default::default());
        let mut b: Box<LargerStruct> = Box::new(Default::default());
        let mut b1: Vec<I32Struct> = Vec::with_capacity(element_count);
        let mut c: Box<MixedStruct> = Box::new(Default::default());
        let mut d: Box<SmallerStruct> = Box::new(Default::default());
        let mut e: SmallStruct = Default::default();
        let mut f: MixedStruct = Default::default();
        let mut g: I32Struct = Default::default();

        for _ in 0..element_count {
            b1.push(I32Struct { x: 0, y: 0 });
        }

        mutate_mixed_data(
            &mut a, &mut b, &mut c, &mut d, &mut e, &mut f, &mut g,
        );
        g.x += b1.len() as i32;
    });
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
