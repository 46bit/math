use std::io::{stdin, Read};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
struct Statement {
    instruction: Instruction,
    condition: Condition,
}

impl Statement {
    fn evaluate(&self, registers: &mut HashMap<String, i64>) {
        if self.condition.decide(registers) {
            self.instruction.perform(registers);
        }
    }
}

impl FromStr for Statement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<_> = s.split_whitespace().collect();
        if tokens.len() != 7 {
            return Err("invalid token string".to_string());
        }
        if tokens[3] != "if" {
            return Err("missing 'if' token".to_string());
        }
        Ok(Statement {
            instruction: tokens[..3].to_vec().join(" ").parse()?,
            condition: tokens[4..].to_vec().join(" ").parse()?,
        })
    }
}

#[derive(Debug)]
struct Instruction {
    register: String,
    operation: Operation,
    operand: i64,
}

impl Instruction {
    fn perform(&self, registers: &mut HashMap<String, i64>) {
        let register = registers.entry(self.register.clone()).or_insert(0);
        self.operation.apply(register, self.operand);
    }
}

impl FromStr for Instruction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<_> = s.split_whitespace().collect();
        Ok(Instruction {
            register: tokens[0].to_string(),
            operation: tokens[1].parse()?,
            operand: tokens[2]
                .parse()
                .map_err(|_| "unable to parse operand".to_string())?,
        })
    }
}

#[derive(Debug)]
enum Operation {
    Increment,
    Decrement,
}

impl Operation {
    fn apply(&self, register: &mut i64, operand: i64) {
        match *self {
            Operation::Increment => *register += operand,
            Operation::Decrement => *register -= operand,
        };
    }
}

impl FromStr for Operation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "inc" => Ok(Operation::Increment),
            "dec" => Ok(Operation::Decrement),
            s => Err(s.to_string()),
        }
    }
}

#[derive(Debug)]
struct Condition {
    register: String,
    comparator: Comparator,
    operand: i64,
}

impl Condition {
    fn decide(&self, registers: &HashMap<String, i64>) -> bool {
        self.comparator.compare(
            registers.get(&self.register).cloned().unwrap_or(0),
            self.operand,
        )
    }
}

impl FromStr for Condition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<_> = s.split_whitespace().collect();
        Ok(Condition {
            register: tokens[0].to_string(),
            comparator: tokens[1].parse()?,
            operand: tokens[2]
                .parse()
                .map_err(|_| "unable to parse operand".to_string())?,
        })
    }
}

#[derive(Debug)]
enum Comparator {
    GreaterThan,
    GreaterThanOrEqualTo,
    LesserThan,
    LesserThanOrEqualTo,
    EqualTo,
    NotEqualTo,
}

impl Comparator {
    fn compare(&self, a: i64, b: i64) -> bool {
        match *self {
            Comparator::GreaterThan => a > b,
            Comparator::GreaterThanOrEqualTo => a >= b,
            Comparator::LesserThan => a < b,
            Comparator::LesserThanOrEqualTo => a <= b,
            Comparator::EqualTo => a == b,
            Comparator::NotEqualTo => a != b,
        }
    }
}

impl FromStr for Comparator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">" => Ok(Comparator::GreaterThan),
            ">=" => Ok(Comparator::GreaterThanOrEqualTo),
            "<" => Ok(Comparator::LesserThan),
            "<=" => Ok(Comparator::LesserThanOrEqualTo),
            "==" => Ok(Comparator::EqualTo),
            "!=" => Ok(Comparator::NotEqualTo),
            s => Err(s.to_string()),
        }
    }
}

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d7a(&input);
    println!("{:?}", output);
}

fn d7a(input: &str) -> i64 {
    let mut registers = HashMap::new();
    let mut max = i64::min_value();
    for line in input.lines() {
        let statement: Statement = match line.parse() {
            Ok(s) => s,
            Err(e) => {
                println!("could not parse because {:?}", e);
                continue;
            }
        };
        //println!("{:?}", statement);
        statement.evaluate(&mut registers);
        //println!("{:?}", registers);
        if let Some(current_max) = registers.values().max().cloned() {
            if current_max > max {
                max = current_max;
            }
        }
    }
    println!("{:?}", registers);
    return max;
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn provided_testcase() {
        let input = "b inc 5 if a > 1
a inc 1 if b < 5
c dec -10 if a >= 1
c inc -20 if c == 10";
        assert_eq!(d7a(input), 1);
    }
}
