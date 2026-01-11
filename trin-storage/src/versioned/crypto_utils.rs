
use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};

const SALT_LENGTH: usize = 16;
const TOKEN_LENGTH: usize = 32;

pub fn generate_salt() -> String {
    let mut rng = thread_rng();
    (0..SALT_LENGTH)
        .map(|_| rng.gen_range(33..127) as u8 as char)
        .collect()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn generate_api_token() -> String {
    let mut rng = thread_rng();
    (0..TOKEN_LENGTH)
        .map(|_| rng.gen_range(48..123) as u8 as char)
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

pub fn validate_password(input: &str, salt: &str, expected_hash: &str) -> bool {
    hash_password(input, salt) == expected_hash
}