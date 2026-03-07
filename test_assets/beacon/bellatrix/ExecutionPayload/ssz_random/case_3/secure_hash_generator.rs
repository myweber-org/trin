use sha2::{Digest, Sha256};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub struct SecureHashGenerator {
    salt_length: usize,
    iterations: u32,
}

impl SecureHashGenerator {
    pub fn new(salt_length: usize, iterations: u32) -> Self {
        SecureHashGenerator {
            salt_length,
            iterations,
        }
    }

    pub fn generate_salt(&self) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(self.salt_length)
            .map(char::from)
            .collect()
    }

    pub fn hash_password(&self, password: &str, salt: &str) -> String {
        let mut combined = format!("{}{}", password, salt).into_bytes();
        
        for _ in 0..self.iterations {
            let mut hasher = Sha256::new();
            hasher.update(&combined);
            combined = hasher.finalize().to_vec();
        }
        
        hex::encode(combined)
    }

    pub fn create_secure_hash(&self, password: &str) -> (String, String) {
        let salt = self.generate_salt();
        let hash = self.hash_password(password, &salt);
        (hash, salt)
    }

    pub fn verify_password(&self, password: &str, hash: &str, salt: &str) -> bool {
        let computed_hash = self.hash_password(password, salt);
        computed_hash == hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let generator = SecureHashGenerator::new(16, 1000);
        let password = "MySecurePassword123!";
        
        let (hash1, salt1) = generator.create_secure_hash(password);
        let (hash2, salt2) = generator.create_secure_hash(password);
        
        assert_ne!(hash1, hash2);
        assert_ne!(salt1, salt2);
        
        assert!(generator.verify_password(password, &hash1, &salt1));
        assert!(generator.verify_password(password, &hash2, &salt2));
        assert!(!generator.verify_password("WrongPassword", &hash1, &salt1));
    }

    #[test]
    fn test_deterministic_hashing() {
        let generator = SecureHashGenerator::new(8, 500);
        let password = "test";
        let salt = "abc123";
        
        let hash1 = generator.hash_password(password, salt);
        let hash2 = generator.hash_password(password, salt);
        
        assert_eq!(hash1, hash2);
    }
}