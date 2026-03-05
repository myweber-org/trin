
use rand::Rng;
use rand::rngs::OsRng;

pub fn generate_secure_token(length: usize) -> Result<String, Box<dyn std::error::Error>> {
    if length == 0 {
        return Err("Token length must be greater than zero".into());
    }
    
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !@#$%^&*()-_=+[]{}|;:,.<>?";
    
    let mut rng = OsRng;
    let mut token = String::with_capacity(length);
    
    for _ in 0..length {
        let idx = rng.gen_range(0..CHARSET.len());
        token.push(CHARSET[idx] as char);
    }
    
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_generation() {
        let token = generate_secure_token(32).unwrap();
        assert_eq!(token.len(), 32);
        
        for ch in token.chars() {
            assert!(ch.is_ascii());
        }
    }
    
    #[test]
    fn test_zero_length() {
        let result = generate_secure_token(0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_different_tokens() {
        let token1 = generate_secure_token(16).unwrap();
        let token2 = generate_secure_token(16).unwrap();
        assert_ne!(token1, token2);
    }
}