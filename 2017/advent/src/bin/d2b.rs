use std::io::{stdin, Read};

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d2b(&input);
    println!("{:?}", output);
}

fn d2b(input: &str) -> u32 {
    matrix(input)
        .into_iter()
        .map(|row| {
            for (i, n) in row.iter().enumerate() {
                for (j, m) in row.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    if (n / m) * m == *n {
                        return n / m;
                    }
                }
            }
            unreachable!();
        })
        .sum()
}

fn matrix(digit_string: &str) -> Vec<Vec<u32>> {
    digit_string
        .lines()
        .map(|line| {
            line.split_whitespace()
                .filter_map(|word| word.parse().ok())
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn provided_testcase() {
        assert_eq!(d2b("5 9 2 8\n 9 4 7 3 \n3 8 6 5"), 9);
    }
}
