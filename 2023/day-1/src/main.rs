use std::fs;
use std::error::Error;
use std::env;

fn get_digit_by_name(slice: &str) -> Option<u32> {
    const DIGITS: &'static [&'static str] = &[
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
    ];

    for (idx, digit) in DIGITS.iter().enumerate() {
        if slice.contains(digit) {
            return Some(u32::try_from(idx + 1).unwrap())
        }
    }
    return None
}

fn get_digits(line: &str) -> u32 {
    let bytes = line.as_bytes();
    let mut first: Option<u32> = None;
    let mut second: Option<u32> = None;
    let mut i = 0;
    let mut j = 0;
    while (first == None || second == None) && i != line.len() && j != line.len() {
        if first == None {
            let c = bytes[i] as char;
            i += 1;
            if let Some(d) = c.to_digit(10) {
                first = Some(d);
            } else if let Some(d) = get_digit_by_name(&line[0..=i]) {
                first = Some(d);
            }
        }
        if second == None {
            let idx = line.len() - 1 - j;
            let c = bytes[idx] as char;
            j += 1;
            if let Some(d) = c.to_digit(10) {
                second = Some(d);
            } else if let Some(d) = get_digit_by_name(&line[idx..line.len()]) {
                second = Some(d);
            }
        }
    }

    format!("{}{}", first.unwrap_or(0), second.unwrap_or(0))
        .parse::<u32>()
        .unwrap_or(0)
}

fn get_file_calibration_value(filename: String) -> Result<u32, Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let sum: u32 = contents
        .lines()
        .map(get_digits)
        .sum();
    Ok(sum)
}

fn main() {
    let mut args = env::args();
    args.next();

    let input_file = args.next().expect("No input file provided");
    match get_file_calibration_value(input_file) {
        Ok(sum) => println!("Sum is: {}", sum),
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}
