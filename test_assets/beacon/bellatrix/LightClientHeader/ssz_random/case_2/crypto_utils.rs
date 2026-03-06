use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_token(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_numeric_code(digits: u32) -> u32 {
    let min = 10u32.pow(digits - 1);
    let max = 10u32.pow(digits) - 1;
    thread_rng().gen_range(min..=max)
}