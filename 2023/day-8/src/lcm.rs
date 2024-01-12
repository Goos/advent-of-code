pub fn gcd(a: u64, b: u64) -> u64 {
    if a == 0 {
        return b;
    }
    gcd(b % a, a)
}

pub fn lcm(a: u64, b: u64) -> u64 {
    a * b / gcd(a, b)
}

pub fn lcm_all(inputs: Vec<u64>) -> u64 {
    inputs.iter().fold(1, |a, b| lcm(a, *b))
}
