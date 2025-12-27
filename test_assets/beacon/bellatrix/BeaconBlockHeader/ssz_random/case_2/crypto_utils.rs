
use rand::Rng;
use sha2::{Digest, Sha256};

pub fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
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
    fn test_generate_random_bytes() {
        let bytes = generate_random_bytes(32);
        assert_eq!(bytes.len(), 32);
        let bytes2 = generate_random_bytes(32);
        assert_ne!(bytes, bytes2);
    }

    #[test]
    fn test_hash_password() {
        let salt = b"random_salt";
        let hash1 = hash_password("my_password", salt);
        let hash2 = hash_password("my_password", salt);
        assert_eq!(hash1, hash2);
        let hash3 = hash_password("different_password", salt);
        assert_ne!(hash1, hash3);
    }
}