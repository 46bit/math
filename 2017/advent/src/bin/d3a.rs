use std::cmp::max;
use std::io::{stdin, Read};

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d3a(&input);
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

fn d3a(input: &str) -> u64 {
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

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn test_2() {
        assert_eq!(r(2), 2);
        assert_eq!(nsew_arounds(r(2)), vec![0, 2, 4, 6]);
        assert_eq!(d3a("2"), 1);
    }

    #[test]
    fn test_3() {
        assert_eq!(r(3), 2);
        assert_eq!(nsew_arounds(r(3)), vec![0, 2, 4, 6]);
        assert_eq!(d3a("3"), 2);
    }

    #[test]
    fn provided_testcase_1() {
        assert_eq!(r(1), 1);
        assert_eq!(d3a("1"), 0);
    }

    #[test]
    fn provided_testcase_2() {
        assert_eq!(r(12), 3);
        assert_eq!(nsew_arounds(r(12)), vec![1, 5, 9, 13]);
        assert_eq!(d3a("12"), 3);
    }

    #[test]
    fn provided_testcase_3() {
        assert_eq!(r(23), 3);
        assert_eq!(d3a("23"), 2);
    }

    #[test]
    fn provided_testcase_4() {
        assert_eq!(d3a("1024"), 31);
    }
}
