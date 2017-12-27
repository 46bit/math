use std::io::{stdin, Read};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
struct Disc {
    name: String,
    weight: u64,
    child_names: Vec<String>,
    parent_name: Option<String>,
}

impl Disc {
    fn net_weight(&self, discs: &HashMap<String, Disc>) -> u64 {
        let mut net_weight = self.weight;
        for child_name in &self.child_names {
            net_weight += discs[child_name].net_weight(&discs);
        }
        return net_weight;
    }
}

fn main() {
    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = d7b(&input);
    println!("{:?}", output);
}

fn d7b(input: &str) -> String {
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

    let mut root_name = None;
    for disc in discs.values() {
        if disc.parent_name.is_none() {
            root_name = Some(disc.name.clone());
        }
    }

    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_front(root_name.unwrap());
    while let Some(disc_name) = queue.pop_back() {
        let discs2 = discs.clone();
        let child_names = discs[&disc_name].child_names.clone();
        let children = child_names
            .clone()
            .into_iter()
            .map(|child_name| discs[&child_name].clone());
        let child_weights: HashSet<u64> = children.map(|c| c.net_weight(&discs2)).collect();
        let children = child_names
            .clone()
            .into_iter()
            .map(|child_name| discs[&child_name].clone());
        let children2 = child_names
            .clone()
            .into_iter()
            .map(|child_name| discs[&child_name].clone());
        if child_weights.len() > 1 {
            println!(
                "{:?} {:?}",
                children.collect::<Vec<_>>(),
                children2.map(|c| c.net_weight(&discs2)).collect::<Vec<_>>()
            );
        }
        queue.extend(child_names.into_iter());
    }
    return "".to_string();
}
