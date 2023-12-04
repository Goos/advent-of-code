use quadtree_rs::{area::{AreaBuilder, Area}, point::Point, Quadtree, iter::Iter};
use std::cmp::max;
use std::env;
use std::fs;

#[derive(Debug)]
pub enum Item {
    Part(String),
    Symbol(char)
}

pub struct ItemMatrix(Quadtree<u32, Item>);
impl ItemMatrix {
    pub fn add_symbol(&mut self, symbol: char, point: Point<u32>) {
        self.0.insert_pt(point, Item::Symbol(symbol));
    }

    pub fn add_part(&mut self, part: String, point: Point<u32>) {
        if let Some(width) = u32::try_from(part.chars().count()).ok() {
            if width == 0 {
                return
            }

            let region = AreaBuilder::default()
                .anchor(point)
                .dimensions((width, 1))
                .build()
                .unwrap();
            self.0.insert(region, Item::Part(part));
        }
    }

    fn has_symbol(&self, area: Area<u32>) -> bool {
        self.0.query(area)
            .any(|entry| matches!(entry.value_ref(), Item::Symbol(_)))
    }

    fn iter(&self) -> Iter<u32, Item> {
        self.0.iter()
    }

    pub fn find_parts(&self, area: Area<u32>) -> Vec<u32> {
        self.0.query(area)
            .filter_map(|entry| {
                match entry.value_ref() {
                    Item::Part(part) => part.parse::<u32>().ok(),
                    Item::Symbol(_) => None
                }
            })
            .collect()
    }

    pub fn find_real_parts(&self) -> Vec<u32> {
        self.iter()
            .filter_map(|entry| {
                match entry.value_ref() {
                    Item::Part(part) => {
                        let area = entry.area();
                        if self.has_symbol(get_surrounding_area(&area)) {
                            Some(part)
                        } else {
                            None
                        }
                    }
                    Item::Symbol(_) => None
                }
            })
            .map(|p| p.parse::<u32>().unwrap())
            .collect()
    }

    pub fn find_gear_ratios(&mut self) -> Vec<u32> {
        self.iter()
            .filter_map(|entry| {
                match entry.value_ref() {
                    Item::Part(_) => None,
                    Item::Symbol('*') => {
                        let surrounding = get_surrounding_area(&entry.area());
                        let parts = self.find_parts(surrounding);
                        if parts.iter().count() == 2 {
                            Some(parts.iter().fold(1, |res, a| res * a))
                        } else {
                            None
                        }
                    }
                    Item::Symbol(_) => None
                }
            })
            .collect()
    }
}

fn get_surrounding_area(area: &Area<u32>) -> Area<u32> {
    let x = if area.left_edge() == 0 { 0 } else { area.left_edge() - 1 };
    let y = if area.top_edge() == 0 { 0 } else { area.top_edge() - 1 };
    let width = if area.left_edge() == 0 { area.width() + 1 } else { area.width() + 2 };
    let height = if area.top_edge() == 0 { area.height() + 1 } else { area.height() + 2 };
    AreaBuilder::default()
        .anchor(Point { x, y })
        .dimensions((width, height))
        .build()
        .unwrap()
}

fn parse(input: &String) -> Result<ItemMatrix, String> {
    let max_x = input.lines().count();
    let max_y = input.lines().next().ok_or("Empty input provided")?.len();
    let depth = f32::sqrt(max(max_x, max_y) as f32) as usize + 1;

    let mut matrix = ItemMatrix(Quadtree::<u32, Item>::new(depth));
    let lines = input.lines().enumerate();
    for (y, line) in lines {
        let mut iter = line.chars().enumerate().peekable();
        while let Some((x, letter)) = iter.next() {
            let point = Point { 
                x: u32::try_from(x).unwrap(),
                y: u32::try_from(y).unwrap()
            };
            if letter == '.' {
                continue
            } else if letter.is_numeric() {
                let mut digits: Vec<char> = vec![letter];
                while let Some((_, l2)) = &iter.peek() {
                    if l2.is_numeric() {
                        digits.push(l2.clone());
                    } else {
                        break
                    }
                    iter.next();
                }
                let str: String = digits.into_iter().collect();
                matrix.add_part(str, point);
            } else {
                matrix.add_symbol(letter, point);
            }
        }
    }

    Ok(matrix)
}

fn main() {
    let mut args = env::args();
    args.next();
    let filename = args.next().expect("No input file provided");
    let contents = fs::read_to_string(filename).expect("Input file could not be read");
    let mut matrix = parse(&contents).expect("Couldn't parse input into matrix");
    let real_parts = matrix.find_real_parts();
    println!("parts: {:?}", real_parts.iter().sum::<u32>());
    let gear_ratios = matrix.find_gear_ratios();
    println!("gear ratios: {:?}", gear_ratios.iter().sum::<u32>());
}
