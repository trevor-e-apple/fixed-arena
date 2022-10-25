use std::vec::Vec;

use test::Bencher;

use crate::test_common::{
    get_element_count, I32Struct, LargerStruct, MixedStruct, SmallStruct,
    SmallerStruct,
};

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
