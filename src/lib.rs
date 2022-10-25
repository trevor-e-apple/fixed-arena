#![feature(test)]
extern crate test;

// TODO: no STD this library

pub mod dynamic_arena;
pub mod errors;
pub mod fixed_arena;
pub mod platform;

#[cfg(test)]
mod bench_bumpalo;
#[cfg(test)]
mod bench_std;
#[cfg(test)]
mod test_common;
