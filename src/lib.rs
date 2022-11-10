#![feature(test)]
extern crate test;

// TODO: no STD this library

pub mod errors;
pub mod fixed_arena;

#[cfg(test)]
mod bench_bumpalo;
#[cfg(test)]
mod bench_std;
#[cfg(test)]
mod test_common;
