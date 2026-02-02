
use rand::Rng;
use rand::rngs::OsRng;

pub fn generate_secure_token(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let mut rng = OsRng;
    let token: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
    token
}

pub fn generate_alphanumeric_token(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    
    let mut rng = OsRng;
    let token: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_length() {
        let token = generate_secure_token(32);
        assert_eq!(token.len(), 32);
        
        let alphanumeric = generate_alphanumeric_token(16);
        assert_eq!(alphanumeric.len(), 16);
    }

    #[test]
    fn test_alphanumeric_charset() {
        let token = generate_alphanumeric_token(100);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_secure_token_uniqueness() {
        let token1 = generate_secure_token(24);
        let token2 = generate_secure_token(24);
        assert_ne!(token1, token2);
    }
}