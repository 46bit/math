use std::io::{stdin, Read};
use std::collections::HashSet;

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    for line in input.lines() {
        let output = d4b(&line);
        println!("{:?}", output);
    }
}

fn d4b(input: &str) -> bool {
    let mut words = HashSet::new();
    for token in input.split_whitespace() {
        let mut canonicalised_token: Vec<char> = token.chars().collect();
        canonicalised_token.sort();
        if words.contains(&canonicalised_token) {
            return false;
        }
        words.insert(canonicalised_token);
    }
    return true;
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn provided_testcase_1() {
        assert_eq!(d4b("abcde fghij"), true);
    }

    #[test]
    fn provided_testcase_2() {
        assert_eq!(d4b("abcde xyz ecdab"), false);
    }

    #[test]
    fn provided_testcase_3() {
        assert_eq!(d4b("a ab abc abd abf abj"), true);
    }

    #[test]
    fn provided_testcase_4() {
        assert_eq!(d4b("iiii oiii ooii oooi oooo"), true);
    }

    #[test]
    fn provided_testcase_5() {
        assert_eq!(d4b("oiii ioii iioi iiio"), false);
    }
}
