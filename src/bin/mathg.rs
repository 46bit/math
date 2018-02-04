extern crate math;
extern crate quickcheck;
extern crate rand;

use rand::StdRng;
use quickcheck::{Arbitrary, StdGen};
use std::env;

fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let size: usize = args.next().unwrap().parse().unwrap();
    eprintln!("Generating a program of size {}", size);

    let rng = StdRng::new().unwrap();
    let mut gen = StdGen::new(rng, size);
    println!("{}", math::Program::arbitrary(&mut gen));
}
