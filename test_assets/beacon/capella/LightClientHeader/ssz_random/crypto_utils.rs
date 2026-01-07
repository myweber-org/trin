use rand::Rng;
use sha2::{Sha256, Digest};

const SALT_LENGTH: usize = 16;

pub fn generate_salt() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..SALT_LENGTH).map(|_| rng.gen()).collect()
}

pub fn hash_password(password: &str, salt: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    hex::encode(result)
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
        let password = "secure_password123";
        let salt = generate_salt();
        let hash1 = hash_password(password, &salt);
        let hash2 = hash_password(password, &salt);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_uniqueness() {
        let password = "same_password";
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        let hash1 = hash_password(password, &salt1);
        let hash2 = hash_password(password, &salt2);
        assert_ne!(hash1, hash2);
    }
}