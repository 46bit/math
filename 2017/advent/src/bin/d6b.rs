use std::io::{stdin, Read};
use std::collections::HashMap;

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d6b(&input);
    println!("{:?}", output);
}

fn d6b(input: &str) -> u64 {
    let mut banks: Vec<u64> = input
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    let bank_count = banks.len();

    let mut opcount = 0;
    let mut observed_banks = HashMap::new();
    while !observed_banks.contains_key(&banks) {
        observed_banks.insert(banks.clone(), opcount);
        let (max_bank_index, mut units_remaining) = banks
            .clone()
            .into_iter()
            .enumerate()
            .rev()
            .max_by(|&(_, x), &(_, y)| x.cmp(&y))
            .unwrap();

        banks[max_bank_index] = 0;
        let mut bank_pointer: usize = (max_bank_index + 1) % bank_count;
        while units_remaining > 0 {
            banks[bank_pointer] += 1;
            units_remaining -= 1;
            bank_pointer = (bank_pointer + 1) % bank_count;
        }
        opcount += 1;
    }

    return opcount - observed_banks[&banks];
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn provided_testcase() {
        assert_eq!(d6b("2 4 1 2"), 4);
    }
}
