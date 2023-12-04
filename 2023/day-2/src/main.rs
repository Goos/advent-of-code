use std::cmp::max;
use std::iter::Peekable;
use std::str::FromStr;
use std::env;
use std::fs;
use strum::EnumString;

/**
 * I'm well aware that writing a full parser for this 
 * isn't really necessary, but I wanted to brush up on
 * parser logic and practice working with iterators.
 */

#[derive(Debug)]
enum Token {
    Colon,
    Color(Color),
    Number(u32),
    Semicolon,
    Newline,
}

#[derive(Debug, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
enum Color {
    Red,
    Green,
    Blue
}

#[derive(Debug)]
struct Game {
    id: u32,
    sets: Vec<RevealSet>,
}

impl Default for Game {
    fn default() -> Game {
        Game {
            id: 0,
            sets: Vec::new()
        }
    }
}

#[derive(Debug)]
struct RevealSet {
    red: u32,
    green: u32,
    blue: u32,
}

impl Default for RevealSet {
    fn default() -> RevealSet {
        RevealSet {
            red: 0,
            green: 0,
            blue: 0,
        }
    }
}

fn get_number<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Option<u32> {
    let mut number = iter.next()?.to_digit(10)?;
    while let Some(digit) = iter.peek().map(|c| c.to_digit(10)).flatten() {
        number = number * 10 + digit;
        iter.next();
    }
    Some(number)
}

fn get_color<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Option<Color> {
    let mut word = iter.next()?.to_string();
    while let Some(letter) = iter.peek() {
        if !letter.is_alphabetic() {
            break;
        }
        word.push(letter.clone());
        iter.next();
    }
    Color::from_str(&word).ok()
}

fn lex(input: &String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = input.chars().peekable();
    while let Some(&c) = iter.peek() {
        match c {
            ':' => {
                tokens.push(Token::Colon);
                iter.next();
            }
            ';' => {
                tokens.push(Token::Semicolon);
                iter.next();
            }
            '0'..='9' => {
                if let Some(num) = get_number(&mut iter) {
                    tokens.push(Token::Number(num));
                }
            }
            'a'..='z' => {
                if let Some(color) = get_color(&mut iter) {
                    tokens.push(Token::Color(color));
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

fn parse(input: &String) -> Vec<Game> {
    let lex_tokens = lex(input);

    let mut games: Vec<Game> = Vec::new();
    let mut iter = lex_tokens.iter().peekable();
    while let Some(_) = iter.peek() {
        games.push(parse_game(&mut iter));
    }
    games
}

fn parse_game<'a, T: Iterator<Item = &'a Token>>(iter: &mut Peekable<T>) -> Game {
    let mut game = Game::default();
    while let Some(token) = iter.peek() {
        match token {
            Token::Number(num) => {
                game.id = num.clone();
                iter.next();
            }
            Token::Colon | Token::Semicolon => {
                iter.next();
                game.sets.push(parse_set(iter));
            }
            Token::Newline => {
                iter.next();
                break
            },
            _ => break
        }
    }
    game
}

fn parse_set<'a, T: Iterator<Item = &'a Token>>(iter: &mut Peekable<T>) -> RevealSet {
    let mut set = RevealSet::default();
    while let Some(token) = iter.peek() {
        match token {
            Token::Number(num) => {
                iter.next();
                if let Some(Token::Color(col)) = iter.peek() {
                    match col {
                        Color::Red => set.red = *num,
                        Color::Blue => set.blue = *num,
                        Color::Green => set.green = *num,
                    }
                }
            }
            Token::Color(_) => _ = iter.next(),
            _ => break
        }
    }
    set
}

fn main() {
    let mut args = env::args();
    args.next();
    
    let available = RevealSet {
        red: 12,
        green: 13,
        blue: 14
    };
    let filename = args.next().expect("No input file provided");
    let contents = fs::read_to_string(filename).expect("Input file could not be read");
    let games = parse(&contents);
    
    let possible_game_ids: Vec<u32> = games
        .iter()
        .filter(|g| {
            let has_impossible_set = g.sets.iter().any(|s| {
                s.red > available.red || s.green > available.green || s.blue > available.blue
            });
            !has_impossible_set
        })
        .map(|g| g.id)
        .collect();

    //println!("possible games: {:?}", possible_games);
    println!("possible games sum: {}", possible_game_ids.iter().sum::<u32>());

    let minimum_sets: Vec<RevealSet> = games
        .iter()
        .map(|g| {
            let mut minimum = RevealSet::default();
            for set in g.sets.iter() {
                minimum.red = max(minimum.red, set.red);
                minimum.green = max(minimum.green, set.green);
                minimum.blue = max(minimum.blue, set.blue);
            }
            minimum
        })
        .collect();
    let sum_of_powers: u32 = minimum_sets.iter()
        .map(|s| s.red * s.green * s.blue)
        .sum();
    //println!("minimum sets: {:?}", minimum_sets);
    println!("sum of powers: {}", sum_of_powers);
}
