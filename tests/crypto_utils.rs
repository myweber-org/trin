
use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};

pub fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()";
    
    let mut rng = thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn verify_password(password: &str, salt: &str, hashed: &str) -> bool {
    hash_password(password, salt) == hashed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_string_length() {
        let random = generate_random_string(16);
        assert_eq!(random.len(), 16);
    }

    #[test]
    fn test_password_hashing() {
        let password = "SecurePass123!";
        let salt = "randomsalt";
        let hashed = hash_password(password, salt);
        
        assert!(verify_password(password, salt, &hashed));
        assert!(!verify_password("wrongpass", salt, &hashed));
    }
}