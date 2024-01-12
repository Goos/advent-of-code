use std::collections::HashMap;
use std::iter::Cycle;

use crate::lcm::lcm_all;

#[derive(Debug)]
pub struct Network {
    pub nodes: HashMap<String, (String, String)>
}

#[derive(Debug, Clone)]
pub enum Step {
    Left,
    Right,
}

impl Network {
    pub fn navigate<'a, F1, F2>(
        &'a self, 
        is_start: F1, 
        is_goal: F2, 
        steps: &'a Vec<Step>
    ) -> Option<u64> 
    where
        F1: Fn(&'a String) -> bool,
        F2: Fn(&'a String) -> bool + Copy
    {
        let matching: Vec<&String> = self.nodes.keys()
            .filter(|k| is_start(k))
            .collect();
        match matching.len() {
            0 => None,
            1 => {
                let mut step_iter = steps.iter().cycle();
                Some(self.navigate_rec(is_goal, matching.first().unwrap(), &mut step_iter, 0))
            },
            _ => {
                let required_steps: Vec<u64> = matching.iter()
                    .map(|m| {
                        let mut step_iter = steps.iter().cycle();
                        self.navigate_imp(m, is_goal, &mut step_iter) as u64
                    })
                    .collect();
                Some(lcm_all(required_steps))
            }
        }
    }

    fn navigate_rec<'a, I, F>(
        &'a self, 
        is_goal: F, 
        current: &'a String,
        step_iter: &mut Cycle<I>,
        steps: u64
    ) -> u64 
    where
        I: Iterator<Item = &'a Step> + Clone,
        F: Fn(&'a String) -> bool + Copy,
    {
        let step = step_iter.next();
        let Some(paths) = self.nodes.get(current) else {
            panic!("Could not find: {}", current);
        };
        let next = match step {
            Some(Step::Left) => &paths.0,
            Some(Step::Right) => &paths.1,
            None => panic!("Unexpected")
        };
        if is_goal(next) {
            steps + 1
        } else {
            self.navigate_rec(is_goal, next, step_iter, steps + 1)
        }
    }


    fn navigate_imp<'a, I, F>(
        &'a self, 
        start: &'a String,
        is_goal: F, 
        step_iter: &mut Cycle<I>
    ) -> u64
    where
        I: Iterator<Item = &'a Step> + Clone,
        F: Fn(&'a String) -> bool,
    {
        let mut steps = 0;
        let mut current: &String = start;

        while !is_goal(current) {
            let step = step_iter.next();
            let Some(paths) = self.nodes.get(current) else {
                panic!("Could not find: {}", current);
            };
            current = match step {
                Some(Step::Left) => &paths.0,
                Some(Step::Right) => &paths.1,
                None => panic!("Unexpected")
            };
            steps = steps + 1;
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_navigation() {
        let network = Network {
            nodes: HashMap::from([
                (String::from("AAA"), (String::from("BBB"), String::from("BBB"))),
                (String::from("BBB"), (String::from("AAA"), String::from("ZZZ"))),
                (String::from("ZZZ"), (String::from("ZZZ"), String::from("ZZZ"))),
            ])
        };

        let steps = vec![Step::Left, Step::Left, Step::Right];
        let navigated_steps = network.navigate(|n| n == "AAA", |n| n == "ZZZ", &steps);
        assert_eq!(navigated_steps, Some(6));
    }
}

