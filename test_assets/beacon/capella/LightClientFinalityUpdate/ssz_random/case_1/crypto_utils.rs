use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_secure_token(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_api_key() -> String {
    let prefix = "sk_live_";
    let random_part = generate_secure_token(32);
    format!("{}{}", prefix, random_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_length() {
        let token = generate_secure_token(16);
        assert_eq!(token.len(), 16);
    }

    #[test]
    fn test_api_key_format() {
        let api_key = generate_api_key();
        assert!(api_key.starts_with("sk_live_"));
        assert_eq!(api_key.len(), 40);
    }

    #[test]
    fn test_token_uniqueness() {
        let token1 = generate_secure_token(16);
        let token2 = generate_secure_token(16);
        assert_ne!(token1, token2);
    }
}