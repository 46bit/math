#![feature(slice_rotate)]

use std::io::{stdin, Read};

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d1(&input);
    println!("{:?}", output);
}

fn d1(digit_string: &str) -> u64 {
    let mut nexts = digits(digit_string);
    nexts.rotate(1);
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
        assert_eq!(d1("1122"), 3);
    }

    #[test]
    fn provided_testcase_2() {
        assert_eq!(d1("1111"), 4);
    }

    #[test]
    fn provided_testcase_3() {
        assert_eq!(d1("1234"), 0);
    }

    #[test]
    fn provided_testcase_4() {
        assert_eq!(d1("91212129"), 9);
    }
}
