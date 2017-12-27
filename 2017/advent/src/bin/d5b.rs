use std::io::{stdin, Read};

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d5b(&input);
    println!("{:?}", output);
}

fn d5b(input: &str) -> u64 {
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
        if instructions[old_instruction_pointer as usize] >= 3 {
            instructions[old_instruction_pointer as usize] -= 1;
        } else {
            instructions[old_instruction_pointer as usize] += 1;
        }
        opcount += 1;
    }
    return opcount;
}
