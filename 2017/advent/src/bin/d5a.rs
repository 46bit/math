use std::io::{stdin, Read};

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d5a(&input);
    println!("{:?}", output);
}

fn d5a(input: &str) -> u64 {
    let mut instructions: Vec<i64> = input
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    let instruction_count = instructions.len() as i64;

    let mut opcount = 0;
    let mut instruction_pointer: i64 = 0;
    while instruction_pointer >= 0 && instruction_pointer < instruction_count {
        let old_instruction_pointer = instruction_pointer;
        instruction_pointer += instructions[instruction_pointer as usize];
        instructions[old_instruction_pointer as usize] += 1;
        opcount += 1;
    }
    return opcount;
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn provided_testcase_1() {
        assert_eq!(d5a("0 3 0 1 -3"), 5);
    }

    #[test]
    fn once() {
        assert_eq!(d5a("1"), 1);
        assert_eq!(d5a("-1"), 1);
    }

    #[test]
    fn twice() {
        assert_eq!(d5a("1 1"), 2);
        assert_eq!(d5a("1 -2"), 2);
    }

    #[test]
    fn incrementing() {
        assert_eq!(d5a("0"), 2);
        assert_eq!(d5a("0 1"), 3);
        assert_eq!(d5a("1 0"), 3);
        assert_eq!(d5a("0 0"), 4);
    }
}
