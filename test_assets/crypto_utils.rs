
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_secure_token(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub fn generate_api_key() -> String {
    let token = generate_secure_token(32);
    format!("sk_{}", token)
}

pub fn generate_session_id() -> String {
    generate_secure_token(64)
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
        let key = generate_api_key();
        assert!(key.starts_with("sk_"));
        assert_eq!(key.len(), 35); // 3 prefix chars + 32 random chars
    }

    #[test]
    fn test_session_id_length() {
        let session_id = generate_session_id();
        assert_eq!(session_id.len(), 64);
    }

    #[test]
    fn test_unique_tokens() {
        let token1 = generate_secure_token(16);
        let token2 = generate_secure_token(16);
        assert_ne!(token1, token2);
    }
}