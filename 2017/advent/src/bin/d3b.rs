use std::cmp::max;
use std::io::{stdin, Read};

// fn neighbours(i: u64) -> Vec<i64> {
//     let ring = r(i);
//     r-1,r,r+1
//     i-1,
// }

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d3b(&input);
    println!("{:?}", output);
}

fn r(i: u64) -> u64 {
    1 + (((i as f64).sqrt() - 1.0) / 2.0).ceil() as u64
}

fn nsew_arounds(r: u64) -> Vec<u64> {
    vec![
        max(0, r - 2),
        (r - 1) * 3 - 1,
        (r - 1) * 5 - 1,
        (r - 1) * 7 - 1,
    ]
}

fn d3b(input: &str) -> u64 {
    let i: u64 = input.trim().parse().unwrap();
    let r = r(i);
    if r == 1 {
        return 0;
    }
    let a = i - (((r - 1) * 2 - 1).pow(2) + 1);

    let m = nsew_arounds(r)
        .into_iter()
        .map(|p| ((a as i64) - (p as i64)).abs() as u64)
        .min()
        .unwrap();
    return m + (r - 1);
}
