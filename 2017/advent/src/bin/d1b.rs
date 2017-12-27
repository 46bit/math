#![feature(slice_rotate)]

use std::io::{stdin, Read};

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d2(&input);
    println!("{:?}", output);
}

fn d2(digit_string: &str) -> u64 {
    let mut nexts = digits(digit_string);
    let nexts_len = nexts.len();
    nexts.rotate(nexts_len / 2);
    let mut total = 0;
    for (index, current) in digits(digit_string).into_iter().enumerate() {
        let next = nexts[index];
        if current == next {
            total += current as u64;
        }
    }
    total
}

fn digits(digit_string: &str) -> Vec<u32> {
    digit_string
        .chars()
        .filter_map(|c| c.to_digit(10))
        .collect()
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn provided_testcase_1() {
        assert_eq!(d2("1212"), 6);
    }

    #[test]
    fn provided_testcase_2() {
        assert_eq!(d2("1221"), 0);
    }

    #[test]
    fn provided_testcase_3() {
        assert_eq!(d2("123425"), 4);
    }

    #[test]
    fn provided_testcase_4() {
        assert_eq!(d2("123123"), 12);
    }

    #[test]
    fn provided_testcase_5() {
        assert_eq!(d2("12131415"), 4);
    }
}
