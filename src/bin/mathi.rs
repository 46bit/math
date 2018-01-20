extern crate langs;

use langs::math;
use std::env;
use std::io::{self, Read};

fn main() {
    let mut in_ = String::new();
    io::stdin().read_to_string(&mut in_).unwrap();

    let mut args = env::args();
    args.next().unwrap();
    let inputs = args.map(|input| input.parse().unwrap()).collect();

    let outputs = math::interpret(in_.as_bytes(), &inputs).unwrap();
    for n in outputs {
        println!("{}", n);
    }
}
