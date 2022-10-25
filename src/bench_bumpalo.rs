use std::mem::size_of;

use test::Bencher;

use bumpalo::Bump;

use crate::test_common::{
    get_element_count, I32Struct, LargerStruct, MixedStruct, SmallStruct,
    SmallerStruct,
};

#[bench]
fn bump_alloc_array(b: &mut Bencher) {
    let element_count = get_element_count();
    let mut arena = Bump::new();

    let initial_value = I32Struct {
        ..Default::default()
    };
    b.iter(|| {
        let elements =
            arena.alloc_slice_fill_clone(element_count, &initial_value);
        for (index, element) in elements.iter_mut().enumerate() {
            element.x = index as i32;
            element.y = -1 * (index as i32);
        }
        arena.reset();
    });
}

#[bench]
fn bump_alloc_prealloc(b: &mut Bencher) {
    let element_count = get_element_count();
    let mut arena = Bump::with_capacity(element_count * size_of::<I32Struct>());

    let initial_value = I32Struct {
        ..Default::default()
    };
    b.iter(|| {
        let elements =
            arena.alloc_slice_fill_clone(element_count, &initial_value);
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
    let initial_value = I32Struct {
        ..Default::default()
    };
    let mut arena = Bump::new();
    b.iter(|| {
        let a = arena.alloc(I32Struct {
            ..Default::default()
        });
        let b = arena.alloc(LargerStruct {
            ..Default::default()
        });
        let b1 = arena.alloc_slice_fill_clone(element_count, &initial_value);
        let c = arena.alloc(MixedStruct {
            ..Default::default()
        });
        let d = arena.alloc(SmallerStruct {
            ..Default::default()
        });
        let e = arena.alloc(SmallStruct {
            ..Default::default()
        });
        let f = arena.alloc(MixedStruct {
            ..Default::default()
        });
        let g = arena.alloc(I32Struct {
            ..Default::default()
        });

        mutate_mixed_data(a, b, c, d, e, f, g);
        g.x += b1.len() as i32;

        arena.reset();
    });
}
