use std::io::{stdin, Read};
use std::collections::HashMap;

#[derive(Debug)]
struct Disc {
    name: String,
    weight: u64,
    child_names: Vec<String>,
    parent_name: Option<String>,
}

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d7a(&input);
    println!("{:?}", output);
}

fn d7a(input: &str) -> String {
    let mut discs = HashMap::new();
    for line in input.lines() {
        let tokens: Vec<_> = line.split_whitespace().collect();
        let mut disc = Disc {
            name: tokens[0].to_string(),
            weight: tokens[1]
                .trim_left_matches('(')
                .trim_right_matches(')')
                .parse()
                .unwrap(),
            child_names: Vec::new(),
            parent_name: None,
        };
        if tokens.len() > 3 && tokens[2] == "->" {
            for mut token in tokens[3..].iter() {
                disc.child_names
                    .push(token.trim_right_matches(',').to_string());
            }
        }
        discs.insert(disc.name.clone(), disc);
    }

    let disc_names: Vec<_> = discs.keys().cloned().collect();
    for disc_name in disc_names {
        let child_names = discs[&disc_name].child_names.clone();
        for child_name in child_names {
            discs.get_mut(&child_name).unwrap().parent_name = Some(disc_name.clone());
        }
    }

    let mut bottom_name = None;
    for disc in discs.values() {
        if disc.parent_name.is_none() {
            bottom_name = Some(disc.name.clone());
        }
    }
    return bottom_name.unwrap();
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn provided_testcase() {
        let input = "pbga (66)
xhth (57)
ebii (61)
havc (66)
ktlj (57)
fwft (72) -> ktlj, cntj, xhth
qoyq (66)
padx (45) -> pbga, havc, qoyq
tknk (41) -> ugml, padx, fwft
jptl (61)
ugml (68) -> gyxo, ebii, jptl
gyxo (61)
cntj (57)";
        assert_eq!(d7a(input), "xhth");
    }
}
