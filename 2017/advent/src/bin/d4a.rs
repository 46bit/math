use std::io::{stdin, Read};
use std::collections::HashSet;

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    for line in input.lines() {
        let output = d4a(&line);
        println!("{:?}", output);
    }
}

fn d4a(input: &str) -> bool {
    let mut words = HashSet::new();
    for token in input.split_whitespace() {
        if words.contains(token) {
            return false;
        }
        words.insert(token);
    }
    return true;
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn provided_testcase_1() {
        assert_eq!(d4a("aa bb cc dd ee"), true);
    }

    #[test]
    fn provided_testcase_2() {
        assert_eq!(d4a("aa bb cc dd aa"), false);
    }

    #[test]
    fn provided_testcase_3() {
        assert_eq!(d4a("aa bb cc dd aaa"), true);
    }

}
