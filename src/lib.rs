// Silence 'core' feature warnings
// for `error:Error` and such
#![feature(core)]
// Silence 'io' feature warnings
// for `BufWriter` and such
#![feature(io)]

extern crate rand;

pub mod error;
pub mod ops;
pub mod vm;

#[test]
fn it_works() {
}
