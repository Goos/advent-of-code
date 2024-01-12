mod network;
mod lcm;

use std::collections::HashMap;
use std::env;
use std::fs;

use crate::network::{Network, Step};

fn parse_network_and_steps(input: &String) -> Option<(Network, Vec<Step>)> {
    let mut lines = input.lines();
    let Some(steps_line) = lines.next() else {
        return None;
    };
    let steps = parse_steps(steps_line);
    let mut network_map: HashMap<String, (String, String)> = HashMap::new();
    while let Some(line) = lines.next() {
        if let Some(map_line) = parse_map_line(line) {
            network_map.insert(map_line.0, map_line.1);
        }
    }
    let network = Network {
        nodes: network_map
    };

    Some((network, steps))
}

fn parse_steps(input: &str) -> Vec<Step> {
    input.chars()
        .filter_map(|c| {
            match c {
                'L' => Some(Step::Left),
                'R' => Some(Step::Right),
                _ => None
            }
        })
        .collect()
}

fn parse_map_line(input: &str) -> Option<(String, (String, String))> {
    let mut split_input = input.split("=");
    let Some(start_split) = split_input.next() else {
        return None;
    };
    let Some(pointers) = split_input.next() else {
        return None;
    };

    let Some(open_paren_idx) = pointers.char_indices().find(|c| c.1 == '(').map(|c| c.0) else {
        return None;
    };

    let Some(close_paren_idx) = pointers.char_indices().find(|c| c.1 == ')').map(|c| c.0) else {
        return None;
    };

    let start = start_split[0..3].to_string();
    let left = pointers[open_paren_idx + 1..open_paren_idx+4].to_string();
    let right = pointers[close_paren_idx-3..close_paren_idx].to_string();
    Some((start, (left, right)))
}

fn main() {
    let mut args = env::args();
    args.next();
    let input = args.next().expect("No input provided");
    let contents = fs::read_to_string(input).expect("Could not read input file");
    let (network, steps) = parse_network_and_steps(&contents).expect("Could not parse input");
    // let num_steps = network.navigate(|n| n == "AAA", |n| n == "ZZZ", &steps);
    // println!("num_steps: {:?}", num_steps);
    let num_steps_multiple = network.navigate(|n| n.ends_with("A"), |n| n.ends_with("Z"), &steps);
    println!("num_steps_multiple: {:?}", num_steps_multiple);
}
