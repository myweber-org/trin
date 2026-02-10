
use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};

const SALT_LENGTH: usize = 16;
const TOKEN_LENGTH: usize = 32;

pub fn generate_salt() -> String {
    let mut rng = thread_rng();
    (0..SALT_LENGTH)
        .map(|_| rng.gen_range(33..=126) as u8 as char)
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
        .map(|_| rng.gen_range(48..=122) as u8 as char)
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salt_length() {
        let salt = generate_salt();
        assert_eq!(salt.len(), SALT_LENGTH);
    }

    #[test]
    fn test_hash_consistency() {
        let password = "secure_password_123";
        let salt = "test_salt";
        let hash1 = hash_password(password, salt);
        let hash2 = hash_password(password, salt);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_token_alphanumeric() {
        let token = generate_api_token();
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
        assert_eq!(token.len(), TOKEN_LENGTH);
    }
}