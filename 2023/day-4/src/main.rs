use std::collections::HashSet;
use std::iter::Peekable;
use std::env;
use std::fs;
use std::cmp::min;

#[derive(Debug)]
enum Token {
    Card(u32),
    Number(u32),
    Pipe,
}

#[derive(Debug)]
#[derive(Clone)]
struct Card {
    number: u32,
    winning_numbers: HashSet<u32>,
    numbers: HashSet<u32>,
}

impl Default for Card {
    fn default() -> Card {
        Card {
            number: 0,
            winning_numbers: HashSet::new(),
            numbers: HashSet::new(),
        }
    }
}

impl Card {
    fn matches(&self) -> usize {
        self.numbers.iter()
            .filter(|n| self.winning_numbers.contains(n))
            .count()
    }
    fn points(&self) -> u32 {
        let matches = self.matches();
        if matches == 0 {
            0
        } else {
            let mut value = 1;
            for _ in 1..matches {
                value = value * 2;
            }
            value
        }
    }
}

fn lex_contents(contents: String) -> Vec<Token> {
    let mut iter = contents.chars().peekable();
    let mut tokens: Vec<Token> = vec![];
    while let Some(c) = iter.peek() {
        match c {
            'C' => {
                if let Some(card_num) = get_card_number(&mut iter) {
                    tokens.push(Token::Card(card_num));
                }
            }
            '0'..='9' => {
                if let Some(num) = get_number(&mut iter) {
                    tokens.push(Token::Number(num));
                }
            }
            '|' => {
                tokens.push(Token::Pipe);
                iter.next();
            }
            _ => _ = iter.next()
        }
    }
    tokens
}

fn get_number<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Option<u32> {
    let mut number = iter.next()?.to_digit(10)?;
    while let Some(digit) = iter.peek().map(|c| c.to_digit(10)).flatten() {
        number = number * 10 + digit;
        iter.next();
    }
    Some(number)
}

fn get_card_number<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Option<u32> {
    const CARD: &str = "Card ";
    let mut card_num = None;
    let mut i = 0;
    while let Some(c) = iter.peek() {
        // Check if prefix is not "Card "
        if i < CARD.len() {
            if CARD.chars().nth(i) != Some(*c) {
                break;
            } else {
                i += 1;
                iter.next();
            }
        } else {
            if *c == ' ' {
                iter.next();
            } else {
                card_num = get_number(iter);
                break
            }
        }
    }
    card_num
}

fn parse_contents(contents: String) -> Vec<Card> {
    let tokens = lex_contents(contents);
    let mut cards: Vec<Card> = vec![];
    let mut iter = tokens.iter().peekable();
    while let Some(token) = iter.peek() {
        match token {
            Token::Card(num) => {
                iter.next();
                cards.push(parse_card(&mut iter, num.clone()));
            }
            _ => _ = iter.next()
        }
    }

    cards
}

fn parse_card<'a, T: Iterator<Item = &'a Token>>(iter: &mut Peekable<T>, num: u32) -> Card {
    let mut card = Card::default();
    card.number = num;
    let mut parsing_winning = true;
    while let Some(token) = iter.peek() {
        match token {
            Token::Card(num) => {
                break
            }
            Token::Number(num) => {
                if parsing_winning {
                    card.winning_numbers.insert(num.clone());
                } else {
                    card.numbers.insert(num.clone());
                }
                iter.next();
            }
            Token::Pipe => {
                parsing_winning = false;
                iter.next();
            }
        }
    }
    card
}

fn get_card_point_total(cards: &[Card]) -> u32 {
    cards
        .iter()
        .map(|c| c.points())
        .sum()
}

fn get_card_copies_total(cards: &[Card]) -> u32 {
    let mut copies: Vec<(&Card, u32)> = cards
        .iter()
        .map(|c| (c, 1))
        .collect();
    
    let mut i = 0;
    while i < copies.len() {
        let (card, instance_count) = copies[i];
        let matches = card.matches();
        if matches > 0 {
            let from = i + 1;
            let to = min(copies.len(), from + matches);
            for j in from..to {
                let (copy, count) = copies[j];
                copies[j] = (copy, count + instance_count);
            }
        }
        i += 1;
    }
    copies
        .iter()
        .map(|(_, count)| count)
        .sum()
}

fn main() {
    let mut args = env::args();
    args.next();
    let filename = args.next().expect("No input file provided");
    let contents = fs::read_to_string(filename).expect("Input file could not be read");
    let cards = parse_contents(contents);
    println!("Card point totals: {}", get_card_point_total(&cards));
    println!("Card copy totals: {}", get_card_copies_total(&cards));

}
