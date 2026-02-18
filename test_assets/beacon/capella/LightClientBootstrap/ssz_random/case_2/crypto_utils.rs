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
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_string_length() {
        let result = generate_random_string(16);
        assert_eq!(result.len(), 16);
    }

    #[test]
    fn test_password_hashing() {
        let password = "secure_pass123";
        let salt = "random_salt";
        let hash1 = hash_password(password, salt);
        let hash2 = hash_password(password, salt);
        assert_eq!(hash1, hash2);
        
        let different_salt = "another_salt";
        let hash3 = hash_password(password, different_salt);
        assert_ne!(hash1, hash3);
    }
}