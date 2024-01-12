use std::collections::HashMap;
use std::env;
use std::fs;
use std::cmp::{max, min, Ord};
use std::iter::Peekable;
use std::ops::Range;
use std::str::FromStr;
use strum::EnumString;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumString)]
#[strum(serialize_all = "lowercase")]
enum ValueKind {
    Seed,
    Soil,
    Fertilizer,
    Water,
    Light,
    Temperature,
    Humidity,
    Location,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Value {
    kind: ValueKind,
    number: u64,
}

#[derive(Debug, Clone)]
struct RangePair {
    source: Range<u64>, 
    target: Range<u64>,
}

impl RangePair {
    fn subrange(&self, range: &Range<u64>) -> Option<RangePair> {
        // checking that the subrange is contained within the source range
        if self.source.start <= range.start && self.source.end >= range.end {
            let start_offset = range.start - self.source.start;
            let range_length = range.end - range.start;
            let target_start = self.target.start + start_offset;
            let target_end = target_start + range_length;
            Some(RangePair { source: range.clone(), target: target_start..target_end })
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct RangeTreeNode {
    range: RangePair,
    max: u64,
    left: Option<Box<RangeTreeNode>>,
    right: Option<Box<RangeTreeNode>>,
}

fn ranges_overlap(r1: &Range<u64>, r2: &Range<u64>) -> bool {
    r1.start <= r2.end && r2.start <= r1.end
}

fn range_intersection(r1: &Range<u64>, r2: &Range<u64>) -> Option<Range<u64>> {
    if ranges_overlap(r1, r2) {
        let start = max(r1.start, r2.start);
        let end = min(r1.end, r2.end);
        Some(start..end)
    } else {
        None
    }
}

impl RangeTreeNode {
    fn new(range: &RangePair) -> RangeTreeNode {
        let max = range.source.end;
        RangeTreeNode { 
            range: range.clone(), 
            max, 
            left: None, 
            right: None 
        }
    }

    fn insert(&mut self, range: &RangePair) {
        if self.max < range.source.end {
            self.max = range.source.end;
        }

        if range.source.start < self.range.source.start {
            if let Some(left) = &mut self.left {
                left.insert(range);
            } else {
                self.left = Some(Box::new(RangeTreeNode::new(range)));
            }
        } else {
            if let Some(right) = &mut self.right {
                right.insert(range);
            } else {
                self.right = Some(Box::new(RangeTreeNode::new(range)));
            }
        }
    }

    fn find_overlapping(&self, range: &RangePair) -> Option<&RangePair> {
        if ranges_overlap(&self.range.source, &range.source) {
            return Some(&self.range);
        }

        if let Some(left) = &self.left {
            if left.max >= range.source.start {
                return left.find_overlapping(range);
            }
        }

        if let Some(right) = &self.right {
            return right.find_overlapping(range);
        }

        None
    }

    fn find_intersections(&self, range: &Range<u64>) -> Vec<RangePair> {
        let mut intersections: Vec<RangePair> = vec![];

        if let Some(intersection) = range_intersection(&self.range.source, range) {
            if let Some(subrange) = self.range.subrange(&intersection) {
                intersections.push(subrange);
            }
        }

        if let Some(left) = &self.left {
            if left.max >= range.start {
                for intersection in left.find_intersections(range) {
                    intersections.push(intersection);
                }
            }
        }

        if let Some(right) = &self.right {
            if right.max >= range.start {
                for intersection in right.find_intersections(range) {
                    intersections.push(intersection);
                }
            }
        }

        intersections
    }

    fn print_traverse(&self) {
        if let Some(left) = &self.left {
            left.print_traverse();
        }
        println!("([{}-{}], max = {})", self.range.source.start, self.range.source.end, self.max);
        if let Some(right) = &self.right {
            right.print_traverse();
        }
    }
}


#[derive(Debug)]
struct RangeMap {
    source_kind: ValueKind,
    target_kind: ValueKind,
    ranges: Vec<RangePair>,
    range_tree: Option<RangeTreeNode>,
}

impl RangeMap {
    fn new(
        source_kind: ValueKind, 
        target_kind: ValueKind, 
        ranges: Vec<RangePair>
    ) -> RangeMap {
        let mut range_tree: Option<RangeTreeNode> = None;
        for range in &ranges {
            if let Some(range_tree) = &mut range_tree {
                range_tree.insert(&range);
            } else {
                range_tree = Some(RangeTreeNode::new(&range));
            }
        }
        RangeMap {
            source_kind,
            target_kind,
            ranges,
            range_tree
        }
    }

    fn value_for(&self, value: &Value) -> Option<Value> {
        if value.kind != self.source_kind {
            return None
        }

        let range_pair = self.ranges.iter().find(|p| p.source.contains(&value.number));
        if let Some(range_pair) = range_pair {
            let offset = value.number - range_pair.source.start;
            let target_number = range_pair.target.start + offset;
            Some(Value { kind: self.target_kind, number: target_number })
        } else {
            Some(Value { kind: self.target_kind, number: value.number })
        }
    }

    fn ranges_for(&self, range: &Range<u64>) -> Vec<Range<u64>> {
        let Some(tree) = &self.range_tree else { return vec![] };
        let mut ranges: Vec<Range<u64>> = vec![];
        let mut intersections = tree.find_intersections(range);
        intersections.sort_by_key(|r| r.source.start);

        // If there are ranges that aren't intersecting, just map them to the same value
        let Some(first) = intersections.first() else { return vec![] };
        // if there's a gap between the start of the range and the first intersection
        if first.source.start > range.start {
            ranges.push(range.start..first.source.start);
        }
        let mut iter = intersections.iter().peekable();
        while let Some(intersection) = iter.next() {
            ranges.push(intersection.target.clone());

            let Some(next) = iter.peek() else { continue };
            // if there's a gap between this intersection and the next
            if intersection.source.end < next.source.start {
                ranges.push(intersection.source.end..next.source.start);
            }
        }
        // if there's a gap between the last intersection and the end of the range
        if let Some(last) = intersections.last() {
            if last.source.end < range.end {
                ranges.push(last.source.end..range.end);
            }
        }

        ranges
    }
}
struct NumberMapper {
    maps_by_source: HashMap<ValueKind, RangeMap>,
}

impl Default for NumberMapper {
    fn default() -> NumberMapper {
        NumberMapper { maps_by_source: HashMap::new() }
    }
}

impl NumberMapper {
    fn insert(&mut self, range_map: RangeMap) {
        self.maps_by_source.insert(range_map.source_kind, range_map);
    }

    fn map(
        &self,
        value: &Value, 
        target_kind: ValueKind
    ) -> Option<Value> {
        let mut mapped = Some(value.clone());
        while mapped != None && mapped.unwrap().kind != target_kind {
            let mapped_val = mapped.unwrap();
            if let Some(range_map) = self.maps_by_source.get(&mapped_val.kind) {
                mapped = range_map.value_for(&mapped_val);
            } else {
                mapped = None;
                break;
            }
        }
        mapped.map(|v| v.clone())
    }

    fn map_range(
        &self,
        range: &Range<u64>,
        source_kind: ValueKind,
        target_kind: ValueKind
    ) -> Vec<Range<u64>> {
        let mut current_kind = source_kind;
        let mut mapped_ranges = vec![range.clone()];
        while !mapped_ranges.is_empty() && current_kind != target_kind {
            let Some(range_map) = self.maps_by_source.get(&current_kind) else { continue };
            println!("mapping ranges:");
            for range in &mapped_ranges {
                println!("\t[{}..{}] ({})", range.start, range.end, range.end - range.start);
            }
            mapped_ranges = mapped_ranges.iter()
                .map(|r| range_map.ranges_for(r))
                .flatten()
                .collect();
            println!("to ranges: \n");
            for range in &mapped_ranges {
                println!("\t[{}..{}] ({})", range.start, range.end, range.end - range.start);
            }
            println!("for kinds: {:?} -> {:?}", current_kind, range_map.target_kind);
            current_kind = range_map.target_kind;
        }
        mapped_ranges
    }
}

#[derive(Debug)]
enum Token {
    Seeds,
    Number(u64),
    Map(ValueKind, ValueKind),
    Newline,
}

impl Token {
    fn as_number(&self) -> Option<u64> {
        match self {
            Token::Number(num) => Some(num.clone()),
            _ => None
        }
    }
}

fn lex_contents(contents: &String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = contents.chars().peekable();
    while let Some(&c) = iter.peek() {
        match c {
            'a'..='z' => {
                if let Some(token) = lex_alphabetical(&mut iter) {
                    tokens.push(token);
                }
            }
            '0'..='9' => {
                if let Some(num) = lex_number(&mut iter) {
                    tokens.push(num);
                }
            }
            '\n' => {
                tokens.push(Token::Newline);
                iter.next();
            }
            _ => _ = iter.next()
        }
    }
    tokens
}

fn lex_alphabetical<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Option<Token> {
    let mut word = iter.next()?.to_string();
    while let Some(letter) = iter.peek() {
        if !letter.is_alphabetic() && letter != &' ' && letter != &'-' {
            break;
        }
        word.push(letter.clone());
        iter.next();
    }

    if word.contains("seeds") {
        Some(Token::Seeds)
    } else if word.contains("map") {
        let mut parts = word.split(' ').next()?.split('-');
        let source = ValueKind::from_str(parts.next()?).ok()?;
        parts.next();
        let destination = ValueKind::from_str(parts.next()?).ok()?;
        Some(Token::Map(source, destination))
    } else {
        None
    }
}

fn lex_number<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Option<Token> {
    let mut number: u64 = iter.next()?.to_digit(10)?.into();
    while let Some(digit) = iter.peek().map(|c| c.to_digit(10)).flatten() {
        number = number * 10 + digit as u64;
        iter.next();
    }
    Some(Token::Number(number))
}

fn parse_contents(contents: &String) -> Option<(Vec<u64>, NumberMapper)> {
    let tokens = lex_contents(&contents);
    let mut iter = tokens.iter().peekable();
    let mut seeds: Option<Vec<u64>> = None;
    let mut number_mapper: Option<NumberMapper> = None;
    while let Some(token) = iter.peek() {
        match token {
            Token::Seeds => seeds = Some(parse_seeds(&mut iter)),
            Token::Map(_, _) => number_mapper = Some(parse_number_mapper(&mut iter)),
            _ => _ = iter.next()
        }
    }
    Some((seeds?, number_mapper?))
}

fn parse_seeds<'a, T: Iterator<Item = &'a Token>>(iter: &mut Peekable<T>) -> Vec<u64> {
    let mut seeds: Vec<u64> = vec![];
    if let Some(Token::Seeds) = iter.next() {
        while let Some(Token::Number(num)) = iter.next() {
            seeds.push(num.clone());
        }
    }
    seeds
}

fn parse_content_ranges(contents: &String) -> Option<(Vec<Range<u64>>, NumberMapper)> {
    let tokens = lex_contents(&contents);
    let mut iter = tokens.iter().peekable();
    let mut seed_ranges: Option<Vec<Range<u64>>> = None;
    let mut number_mapper: Option<NumberMapper> = None;
    while let Some(token) = iter.peek() {
        match token {
            Token::Seeds => seed_ranges = Some(parse_seed_ranges(&mut iter)),
            Token::Map(_, _) => number_mapper = Some(parse_number_mapper(&mut iter)),
            _ => _ = iter.next()
        }
    }
    Some((seed_ranges?, number_mapper?))
}

fn parse_seed_ranges<'a, T: Iterator<Item = &'a Token>>(iter: &mut Peekable<T>) -> Vec<Range<u64>> {
    let mut seed_ranges: Vec<Range<u64>> = vec![];
    if let Some(Token::Seeds) = iter.next() {
        let mut range_start: Option<u64> = None;
        while let Some(Token::Number(num)) = iter.next() {
            match range_start {
                None => {
                    range_start = Some(num.clone());
                }
                Some(start) => {
                    seed_ranges.push(start..(start + num.clone()));
                    range_start = None;
                }
            }
        }
    }
    seed_ranges
}

fn parse_number_mapper<'a, T: Iterator<Item = &'a Token>>(iter: &mut Peekable<T>) -> NumberMapper {
    let mut number_mapper = NumberMapper::default();
    while let Some(token) = iter.peek() {
        match token {
            Token::Map(source, target) => {
                iter.next();
                iter.next();
                if let Some(range_map) = parse_range_map(iter, source, target) {
                    number_mapper.insert(range_map);
                }
            },
            _ => _ = iter.next()
        }
    }
    number_mapper
}

fn parse_range_map<'a, T: Iterator<Item = &'a Token>>(
    iter: &mut Peekable<T>, 
    source_kind: &ValueKind, 
    target_kind: &ValueKind
) -> Option<RangeMap> {
    let mut range_pairs: Vec<RangePair> = vec![];
    while let Some(token) = iter.peek() {
        match token {
            Token::Number(_) => {
                let target_start = iter.next()?.as_number()?;
                let source_start = iter.next()?.as_number()?;
                let offset = iter.next()?.as_number()?;

                let source = source_start..(source_start + offset);
                let target = target_start..(target_start + offset);
                range_pairs.push(RangePair { source, target });
            }
            Token::Newline => _ = iter.next(),
            _ => break,
        }
    }

    Some(RangeMap::new(source_kind.clone(), target_kind.clone(), range_pairs))
}

fn find_smallest_location(seeds: Vec<u64>, mapper: &NumberMapper) -> Option<u64> {
    seeds
        .iter()
        .filter_map(|s| {
            let value = Value { kind: ValueKind::Seed, number: s.clone() }; 
            let result = mapper.map(&value, ValueKind::Location);
            result.map(|r| r.number)
        })
        .min()
}

fn find_smallest_location_ranges(seed_ranges: Vec<Range<u64>>, mapper: &NumberMapper) -> Option<u64> {
    seed_ranges
        .iter()
        .map(|r| mapper.map_range(r, ValueKind::Seed, ValueKind::Location))
        .flatten()
        .map(|r| r.start)
        .min()
}

fn main() {
    let mut args = env::args();
    args.next();
    let input = args.next().expect("No input provided");
    let use_ranges = if let Some(config) = args.next() {
        config == "--ranges"
    } else {
        false
    };
    let contents = fs::read_to_string(input).expect("Could not read input file.");
    if use_ranges {
        let (seed_ranges, mapper) = parse_content_ranges(&contents).expect("Could not parse input");
        let smallest_location = find_smallest_location_ranges(seed_ranges, &mapper)
            .expect("Couldn't map any seeds to locations");
        println!("smallest location: {}", smallest_location)
    } else {
        let (seeds, mapper) = parse_contents(&contents).expect("Could not parse input");
        let smallest_location = find_smallest_location(seeds, &mapper)
            .expect("Couldn't map any seeds to locations");
        println!("smallest location: {}", smallest_location)
    }
}

#[test]
fn range_map_test() {
    let mut source: Range<u64> = 1..2;
    let mut target:  Range<u64> = 4..6;

    let map = RangeMap {
        source_kind: ValueKind::Seed,
        target_kind: ValueKind::Soil,
        ranges: vec![RangePair { source, target }],
        range_tree: None
    };
    let seed = Value { kind: ValueKind::Seed, number: 1 };
    let soil = map.value_for(&seed).unwrap();
    assert_eq!(soil.number, 4);
}

#[test]
fn value_mapper_test() {
    let seeds_1: Range<u64> = 1..2;
    let soils_1: Range<u64> = 4..6;
    let seeds_2: Range<u64> = 5..7;
    let soils_2: Range<u64> = 7..9;
    let humidities: Range<u64> = 9..10;

    let seed_to_soil = RangeMap {
        source_kind: ValueKind::Seed,
        target_kind: ValueKind::Soil,
        ranges: vec![
            RangePair { source: seeds_1.clone(), target: soils_1.clone() },
            RangePair { source: seeds_2.clone(), target: soils_2.clone() }
        ],
        range_tree: None
    };
    let soil_to_humidity = RangeMap {
        source_kind: ValueKind::Soil,
        target_kind: ValueKind::Humidity,
        ranges: vec![
            RangePair { source: soils_1.clone(), target: humidities.clone() }
        ],
        range_tree: None
    };
    let mut mapper = NumberMapper::default();
    mapper.insert(seed_to_soil);
    mapper.insert(soil_to_humidity);
    let humidity = mapper.map(&Value { kind: ValueKind::Seed, number: 1 }, ValueKind::Humidity).unwrap();
    assert_eq!(humidity.number, 9);
    let soil = mapper.map(&Value { kind: ValueKind::Seed, number: 5 }, ValueKind::Soil).unwrap();
    assert_eq!(soil.number, 7);
}

#[test]
fn parse_contents_test() {
    let root_path = env!("CARGO_MANIFEST_DIR");
    let input_file = format!("{}/input.txt", root_path);
    let contents = fs::read_to_string(input_file).expect("Could not read input file.");
    let (seeds, mapper) = parse_contents(&contents).expect("Could not parse input");
    let smallest_location = find_smallest_location(seeds, &mapper)
        .expect("Couldn't map any seeds to locations");
    println!("smallest: {}", smallest_location);
}

#[test]
fn parse_content_ranges_test() {
    let root_path = env!("CARGO_MANIFEST_DIR");
    let input_file = format!("{}/input.txt", root_path);
    let contents = fs::read_to_string(input_file).expect("Could not read input file.");
    let (seed_ranges, mapper) = parse_content_ranges(&contents).expect("Could not parse input");
    let smallest_location = find_smallest_location_ranges(seed_ranges, &mapper)
        .expect("Couldn't map any seeds to locations");
    println!("smallest: {}", smallest_location);
}

#[test]
fn interval_tree_test() {
    let intervals = vec![
        RangePair { source: 100..200, target: 50..150 },
        RangePair { source: 32..48, target: 62..78 },
        RangePair { source: 10..20, target: 90..100 },
        RangePair { source: 255..260, target: 100..105 },
        RangePair { source: 400..420, target: 800..820 },
    ];
    let mut iter = intervals.iter();
    let mut root = RangeTreeNode::new(iter.next().unwrap());
    while let Some(interval) = iter.next() {
        root.insert(interval);
    }
    root.print_traverse();

    //let overlapping1 = root.find_overlapping(&(33..100)).unwrap();
    //assert_eq!(*overlapping1, 0..100);

    //let overlapping2 = root.find_overlapping(&(135..136)).unwrap();
    //assert_eq!(*overlapping2, 120..220);

    let intersections = root.find_intersections(&(120..300));
    println!("intersections: {:?}", intersections);
}
