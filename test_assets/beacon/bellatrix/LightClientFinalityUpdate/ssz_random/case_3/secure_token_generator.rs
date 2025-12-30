use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_api_token(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub fn generate_secure_token() -> String {
    let token = generate_api_token(32);
    format!("sk_{}", token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_length() {
        let token = generate_api_token(16);
        assert_eq!(token.len(), 16);
    }

    #[test]
    fn test_secure_token_format() {
        let token = generate_secure_token();
        assert!(token.starts_with("sk_"));
        assert_eq!(token.len(), 35); // 3 prefix + 32 chars
    }

    #[test]
    fn test_token_uniqueness() {
        let token1 = generate_secure_token();
        let token2 = generate_secure_token();
        assert_ne!(token1, token2);
    }
}