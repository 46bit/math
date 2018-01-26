extern crate langs;

use langs::math;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let in_path = args.next().unwrap();
    let out_path = args.next().unwrap();
    println!("Compiling {} into {}", in_path, out_path);

    let mut in_file = File::open(in_path).unwrap();
    let mut in_ = String::new();
    in_file.read_to_string(&mut in_).unwrap();

    let ir = math::compile(in_.as_bytes(), out_path.clone()).unwrap();
    println!("{}", ir);
}
