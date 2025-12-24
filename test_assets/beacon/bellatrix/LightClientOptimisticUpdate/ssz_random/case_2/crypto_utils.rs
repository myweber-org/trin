
use rand::Rng;

pub fn generate_secure_token(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let mut rng = rand::thread_rng();
    let token: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
    token
}

pub fn generate_api_key() -> String {
    let token = generate_secure_token(32);
    format!("sk_{}", token)
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
        assert!(api_key.starts_with("sk_"));
        assert_eq!(api_key.len(), 35); // sk_ + 32 chars
    }
}