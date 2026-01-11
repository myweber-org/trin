use rand::Rng;
use std::error::Error;

pub fn generate_secure_key(length: usize) -> Result<String, Box<dyn Error>> {
    if length < 16 {
        return Err("Key length must be at least 16 bytes".into());
    }
    
    let mut rng = rand::thread_rng();
    let key: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
    
    Ok(hex::encode(key))
}

pub fn generate_key_pair() -> (String, String) {
    let mut rng = rand::thread_rng();
    let private_key: [u8; 32] = rng.gen();
    let public_key: [u8; 64] = rng.gen();
    
    (hex::encode(private_key), hex::encode(public_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_key() {
        let key = generate_secure_key(32).unwrap();
        assert_eq!(key.len(), 64); // 32 bytes * 2 hex chars
    }

    #[test]
    fn test_key_length_validation() {
        let result = generate_secure_key(8);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_pair_generation() {
        let (private, public) = generate_key_pair();
        assert_eq!(private.len(), 64);
        assert_eq!(public.len(), 128);
    }
}